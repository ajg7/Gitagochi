//! Running a child process with capped output, a timeout and cancellation.

use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// Bytes kept per output stream; the rest is read and dropped.
pub const OUTPUT_CAP: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    /// Ended without an exit code.
    Killed,
    TimedOut,
    Cancelled,
}

#[derive(Debug)]
pub struct RunOut {
    pub exit: Exit,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl RunOut {
    pub fn success(&self) -> bool {
        self.exit == Exit::Code(0)
    }
}

/// A command whose stdin is null, so it cannot read the terminal.
pub fn command(program: &str) -> Command {
    let mut c = Command::new(program);
    c.stdin(Stdio::null());
    c
}

/// Run `cmd`, killing it after `timeout` or when `cancel` is set.
/// Stdin is left as the caller set it (see [`command`]).
pub fn run_capped(cmd: &mut Command, timeout: Duration, cancel: &AtomicBool) -> io::Result<RunOut> {
    run_with_cap(cmd, timeout, cancel, OUTPUT_CAP)
}

type Buf = Arc<Mutex<Vec<u8>>>;

fn reader(src: Option<impl Read + Send + 'static>, cap: usize, done: mpsc::Sender<()>) -> Buf {
    let buf: Buf = Arc::default();
    let out = Arc::clone(&buf);
    thread::spawn(move || {
        if let Some(mut src) = src {
            let mut chunk = [0u8; 8192];
            while let Ok(n) = src.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                if let Ok(mut b) = out.lock() {
                    let room = cap.saturating_sub(b.len()).min(n);
                    b.extend_from_slice(&chunk[..room]);
                }
            }
        }
        let _ = done.send(());
    });
    buf
}

fn take(b: &Buf) -> Vec<u8> {
    b.lock()
        .map(|mut v| std::mem::take(&mut *v))
        .unwrap_or_default()
}

fn stop(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn run_with_cap(
    cmd: &mut Command,
    timeout: Duration,
    cancel: &AtomicBool,
    cap: usize,
) -> io::Result<RunOut> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let (tx, rx) = mpsc::channel();
    let out = reader(child.stdout.take(), cap, tx.clone());
    let err = reader(child.stderr.take(), cap, tx);
    let deadline = Instant::now() + timeout;
    let exit = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code().map_or(Exit::Killed, Exit::Code),
            Ok(None) => {}
            Err(e) => {
                stop(&mut child);
                return Err(e);
            }
        }
        if cancel.load(Ordering::Relaxed) {
            stop(&mut child);
            break Exit::Cancelled;
        }
        if Instant::now() >= deadline {
            stop(&mut child);
            break Exit::TimedOut;
        }
        thread::sleep(Duration::from_millis(50));
    };
    // A surviving grandchild may hold a pipe open, so readers are never joined.
    let grace = if matches!(exit, Exit::Code(_)) {
        5000
    } else {
        200
    };
    let until = Instant::now() + Duration::from_millis(grace);
    for _ in 0..2 {
        let left = until.saturating_duration_since(Instant::now());
        if rx.recv_timeout(left).is_err() {
            break;
        }
    }
    Ok(RunOut {
        exit,
        stdout: take(&out),
        stderr: take(&err),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrn_27_timeout_kills() {
        let never = AtomicBool::new(false);
        let mut c = Command::new("git");
        c.args(["hash-object", "--stdin"]).stdin(Stdio::piped());
        let t = Instant::now();
        let r = run_capped(&mut c, Duration::from_secs(1), &never).unwrap();
        assert_eq!(r.exit, Exit::TimedOut);
        assert!(t.elapsed() < Duration::from_secs(2));

        let r = run_capped(
            command("git").arg("--version"),
            Duration::from_secs(30),
            &never,
        )
        .unwrap();
        assert!(r.success());
        assert!(String::from_utf8_lossy(&r.stdout).contains("git version"));

        let r = run_with_cap(
            command("git").arg("--version"),
            Duration::from_secs(30),
            &never,
            5,
        )
        .unwrap();
        assert_eq!(r.stdout, b"git v");
    }

    #[test]
    fn cancel_stops_child() {
        let cancel = AtomicBool::new(true);
        let mut c = Command::new("git");
        c.args(["hash-object", "--stdin"]).stdin(Stdio::piped());
        let r = run_capped(&mut c, Duration::from_secs(30), &cancel).unwrap();
        assert_eq!(r.exit, Exit::Cancelled);
    }
}
