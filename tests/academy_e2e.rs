//! End-to-end checks that spawn the real binary with GITAGOCHI_DATA_DIR and a temp cwd.
//! The TUI needs a terminal for keys, so `l` opening the learn screen stays manual.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Run the binary up to `ms`; returns the exit status if it exited by itself, plus stdout+stderr.
fn run_for(args: &[&str], cwd: &Path, data: &Path, ms: u64) -> (Option<ExitStatus>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitagochi"))
        .args(args)
        .current_dir(cwd)
        .env("GITAGOCHI_DATA_DIR", data)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (mut out, mut err) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
    let t_out = std::thread::spawn(move || {
        let mut b = Vec::new();
        std::io::Read::read_to_end(&mut out, &mut b).ok();
        b
    });
    let t_err = std::thread::spawn(move || {
        let mut b = Vec::new();
        std::io::Read::read_to_end(&mut err, &mut b).ok();
        b
    });
    let end = Instant::now() + Duration::from_millis(ms);
    let mut status = None;
    while Instant::now() < end {
        if let Some(s) = child.try_wait().unwrap() {
            status = Some(s);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if status.is_none() {
        child.kill().ok();
    }
    child.wait().ok();
    let mut text = String::from_utf8_lossy(&t_out.join().unwrap()).into_owned();
    text.push_str(&String::from_utf8_lossy(&t_err.join().unwrap()));
    (status, text)
}

/// Drop ANSI escape sequences (as spaces) and collapse whitespace.
fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for n in chars.by_ref() {
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

fn files(dir: &Path) -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut v = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let m = e.metadata().unwrap();
            if m.is_dir() {
                stack.push(e.path());
            } else {
                v.push((e.path(), m.modified().unwrap()));
            }
        }
    }
    v.sort();
    v
}

#[test]
fn lrn_29_a_no_paths_no_repo_opens_with_notice() {
    let (cwd, data) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (status, out) = run_for(&[], cwd.path(), data.path(), 2000);
    assert!(status.is_none(), "app exited early: {status:?}\n{out}");
    assert!(
        plain(&out).contains("no repos watched yet"),
        "notice missing: {}",
        plain(&out)
    );
}

#[test]
fn lrn_29_b_explicit_plain_folder_exits_1() {
    let (cwd, data) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let plain_dir = tempfile::tempdir().unwrap();
    let p = plain_dir.path().to_str().unwrap();
    let (status, out) = run_for(&[p], cwd.path(), data.path(), 10_000);
    assert_eq!(status.expect("must exit").code(), Some(1), "{out}");
}

#[test]
fn lrn_30_a_files_land_under_data_dir_only() {
    let platform = dirs::data_dir().unwrap().join("gitagochi");
    let before = files(&platform);
    let (repo, cwd, data) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["commit", "-q", "--allow-empty", "-m", "c1"]);
    let r = repo.path().to_str().unwrap();
    let (status, out) = run_for(
        &["--author", "t@example.com", r],
        cwd.path(),
        data.path(),
        2000,
    );
    assert!(status.is_none(), "app exited early: {status:?}\n{out}");
    assert!(
        data.path().join("pet.json").exists(),
        "pet.json not in data dir"
    );
    assert_eq!(files(&platform), before, "platform data dir was touched");
    assert_eq!(files(cwd.path()), vec![], "cwd was written to");
}
