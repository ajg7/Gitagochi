//! Grading the learner's last commit with hidden tests, fmt and clippy.

use super::curriculum::{Check, CheckKind, MODULES, Module, ObjId, checks_upto};
use super::proc::{Exit, RunOut, command, run_capped};
use crate::git::{self, Repo, strip_control};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const BUILD_TIMEOUT: Duration = Duration::from_secs(300);
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const MESSAGE_MAX_LINES: usize = 40;
const PROBE_TIMEOUT: Duration = Duration::from_secs(60);
const NO_BUILD: &str = "project does not build; fix the build check first";

/// Where and how grading runs.
#[derive(Debug, Clone)]
pub struct GradeEnv {
    pub cargo: String,
    pub academy_dir: PathBuf,
    pub target_dir: PathBuf,
    pub build_timeout: Duration,
    pub test_timeout: Duration,
}

impl GradeEnv {
    pub fn new(data_dir: &Path) -> GradeEnv {
        let academy_dir = data_dir.join("academy");
        GradeEnv {
            cargo: "cargo".into(),
            target_dir: academy_dir.join("target"),
            academy_dir,
            build_timeout: BUILD_TIMEOUT,
            test_timeout: TEST_TIMEOUT,
        }
    }
}

#[derive(Debug)]
pub enum GradeError {
    FolderMissing(PathBuf),
    NotCargo(PathBuf),
    NotARepo(PathBuf),
    NoCommits(PathBuf),
    NotCommitted,
    ToolMissing {
        tool: &'static str,
        hint: &'static str,
    },
    Git(String),
    Io(String),
    Cancelled,
}

impl fmt::Display for GradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GradeError::FolderMissing(p) => write!(
                f,
                "project folder {} does not exist; set up a project with `p`",
                p.display()
            ),
            GradeError::NotCargo(p) => {
                write!(f, "{} is not a cargo project (no Cargo.toml)", p.display())
            }
            GradeError::NotARepo(p) => write!(
                f,
                "{} is not a git repository; run `git init` there",
                p.display()
            ),
            GradeError::NoCommits(p) => write!(
                f,
                "{} has no commits yet; commit your work to grade it",
                p.display()
            ),
            GradeError::NotCommitted => write!(
                f,
                "Cargo.toml is not in your last commit; `git add` and commit it"
            ),
            GradeError::ToolMissing { tool, hint } => write!(
                f,
                "`{tool}` was not found; install it with {hint}. Lessons and quizzes still work."
            ),
            GradeError::Git(e) => write!(f, "git failed: {e}"),
            GradeError::Io(e) => write!(f, "could not prepare the check: {e}"),
            GradeError::Cancelled => write!(f, "check cancelled"),
        }
    }
}

impl std::error::Error for GradeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub module: u8,
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub tags: Vec<ObjId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub commit: String,
    pub results: Vec<CheckResult>,
}

impl Report {
    pub fn passed(&self) -> bool {
        !self.results.is_empty() && self.results.iter().all(|r| r.passed)
    }
}

type Outcome = Result<(), String>;

fn io_err(e: io::Error) -> GradeError {
    GradeError::Io(e.to_string())
}

fn clean(text: &str) -> String {
    let lines: Vec<String> = text
        .lines()
        .map(strip_control)
        .take(MESSAGE_MAX_LINES)
        .collect();
    let mut s = lines.join("\n").trim().to_string();
    if text.lines().count() > MESSAGE_MAX_LINES {
        s.push_str("\n...");
    }
    s
}

fn json_lines(out: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(out)
        .lines()
        .filter(|l| l.starts_with('{'))
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn errors(msgs: &[Value]) -> impl Iterator<Item = &Value> {
    msgs.iter().filter(|v| {
        v["reason"] == "compiler-message"
            && v["message"]["level"] == "error"
            && !v["message"]["message"]
                .as_str()
                .unwrap_or("")
                .starts_with("aborting due to")
    })
}

fn rendered(msgs: &[Value], with_code: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    for v in errors(msgs) {
        let m = &v["message"];
        let text = m["rendered"].as_str().unwrap_or("").trim_end();
        let text = match m["code"]["code"].as_str() {
            Some(code) if with_code && !text.contains(code) => format!("{code}\n{text}"),
            _ => text.to_string(),
        };
        if !out.contains(&text) {
            out.push(text);
        }
    }
    out.join("\n")
}

fn tail(out: &RunOut) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

struct Grader<'a> {
    env: &'a GradeEnv,
    dest: PathBuf,
    cancel: &'a AtomicBool,
}

impl Grader<'_> {
    fn cargo(&self, args: &[&str]) -> Command {
        let mut c = command(&self.env.cargo);
        c.args(args)
            .current_dir(&self.dest)
            .env("CARGO_TARGET_DIR", &self.env.target_dir)
            .env("CARGO_TERM_COLOR", "never");
        c
    }

    fn run(&self, mut c: Command, timeout: Duration) -> Result<RunOut, GradeError> {
        let out = run_capped(&mut c, timeout, self.cancel).map_err(io_err)?;
        if out.exit == Exit::Cancelled {
            return Err(GradeError::Cancelled);
        }
        Ok(out)
    }

    fn timed_out(&self, t: Duration) -> String {
        format!("timed out after {}s (infinite loop?)", t.as_secs())
    }

    fn build(&self) -> Result<Outcome, GradeError> {
        let t = self.env.build_timeout;
        let out = self.run(self.cargo(&["build", "--message-format=json"]), t)?;
        Ok(match out.exit {
            Exit::Code(0) => Ok(()),
            Exit::TimedOut => Err(self.timed_out(t)),
            _ => {
                let r = rendered(&json_lines(&out.stdout), false);
                Err(if r.is_empty() { tail(&out) } else { r })
            }
        })
    }

    fn fmt(&self) -> Result<Outcome, GradeError> {
        let t = self.env.build_timeout;
        let out = self.run(self.cargo(&["fmt", "--check"]), t)?;
        Ok(match out.exit {
            Exit::Code(0) => Ok(()),
            Exit::TimedOut => Err(self.timed_out(t)),
            _ => Err(format!(
                "run `cargo fmt`, then commit:\n{}{}",
                String::from_utf8_lossy(&out.stdout),
                tail(&out)
            )),
        })
    }

    fn clippy(&self) -> Result<Outcome, GradeError> {
        let t = self.env.build_timeout;
        let args = [
            "clippy",
            "--all-targets",
            "--message-format=json",
            "--",
            "-D",
            "warnings",
        ];
        let out = self.run(self.cargo(&args), t)?;
        Ok(match out.exit {
            Exit::Code(0) => Ok(()),
            Exit::TimedOut => Err(self.timed_out(t)),
            _ => {
                let r = rendered(&json_lines(&out.stdout), true);
                Err(if r.is_empty() { tail(&out) } else { r })
            }
        })
    }

    /// Compile module `m`'s hidden tests and run them; returns per-test outcomes.
    fn tests(&self, m: u8, names: &[&str]) -> Result<HashMap<String, Outcome>, GradeError> {
        let target = format!("academy_m{m:02}");
        let t = self.env.build_timeout;
        let c = self.cargo(&[
            "test",
            "--no-run",
            "--message-format=json",
            "--test",
            &target,
        ]);
        let out = self.run(c, t)?;
        let msgs = json_lines(&out.stdout);
        let all = |msg: String| {
            names
                .iter()
                .map(|n| (n.to_string(), Err(msg.clone())))
                .collect()
        };
        if out.exit == Exit::TimedOut {
            return Ok(all(self.timed_out(t)));
        }
        if !out.success() {
            let mut lines: Vec<String> = Vec::new();
            for v in errors(&msgs) {
                let s = v["message"]["message"].as_str().unwrap_or("").to_string();
                if !lines.contains(&s) {
                    lines.push(s);
                }
            }
            let detail = if lines.is_empty() {
                "the hidden tests could not be built".to_string()
            } else {
                lines.join("\n")
            };
            return Ok(all(format!(
                "the hidden tests do not compile against your library:\n{detail}"
            )));
        }
        let exe = msgs.iter().find_map(|v| {
            let is_test = v["reason"] == "compiler-artifact"
                && v["profile"]["test"] == true
                && v["target"]["name"] == target.as_str()
                && v["target"]["kind"]
                    .as_array()
                    .is_some_and(|k| k.iter().any(|x| x == "test"));
            is_test.then(|| v["executable"].as_str().map(PathBuf::from))?
        });
        let Some(exe) = exe else {
            return Ok(all("the hidden test program was not produced".into()));
        };
        let tt = self.env.test_timeout;
        let start = std::time::Instant::now();
        let mut res = Vec::new();
        for n in names {
            let left = tt.saturating_sub(start.elapsed());
            if left.is_zero() {
                res.push((n.to_string(), Err(self.timed_out(tt))));
                continue;
            }
            let mut c = command(&exe.to_string_lossy());
            c.args(["--exact", n, "--test-threads=1", "--color", "never"])
                .current_dir(&self.dest)
                .env("RUST_BACKTRACE", "0");
            let out = self.run(c, left)?;
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            let r = if out.exit == Exit::TimedOut {
                Err(self.timed_out(tt))
            } else if out.success() {
                Ok(())
            } else {
                Err(failure_text(&text, n)
                    .unwrap_or_else(|| "the test did not report a result".into()))
            };
            res.push((n.to_string(), r));
        }
        Ok(res.into_iter().collect())
    }
}

/// The failure output libtest printed for test `name`, without the backtrace hint.
fn failure_text(out: &str, name: &str) -> Option<String> {
    let head = format!("---- {name} stdout ----");
    let mut lines = out.lines().skip_while(|l| *l != head);
    lines.next()?;
    let body: Vec<&str> = lines
        .take_while(|l| !l.starts_with("---- ") && *l != "failures:")
        .filter(|l| !l.starts_with("note: run with `RUST_BACKTRACE"))
        .filter(|l| !(l.starts_with("thread '") && l.contains("academy_m")))
        .collect();
    let s = body.join("\n").trim().to_string();
    Some(if s.is_empty() {
        "the test failed".into()
    } else {
        s
    })
}

fn probe(
    env: &GradeEnv,
    args: &[&str],
    tool: &'static str,
    hint: &'static str,
) -> Result<(), GradeError> {
    let never = AtomicBool::new(false);
    let mut c = command(&env.cargo);
    c.args(args);
    match run_capped(&mut c, PROBE_TIMEOUT, &never) {
        Ok(o) if o.success() => Ok(()),
        _ => Err(GradeError::ToolMissing { tool, hint }),
    }
}

/// Grade the last commit of `project` against modules `1..=upto` of `modules`.
pub fn grade(
    project: &Path,
    modules: &[Module],
    upto: usize,
    env: &GradeEnv,
    cancel: &AtomicBool,
) -> Result<Report, GradeError> {
    if !project.is_dir() {
        return Err(GradeError::FolderMissing(project.to_path_buf()));
    }
    if !project.join("Cargo.toml").is_file() {
        return Err(GradeError::NotCargo(project.to_path_buf()));
    }
    let repo = Repo::open(project).map_err(|e| match e {
        git::GitError::NotARepo(_) => GradeError::NotARepo(project.to_path_buf()),
        other => GradeError::Git(other.to_string()),
    })?;
    if repo.head().is_none() {
        return Err(GradeError::NoCommits(project.to_path_buf()));
    }
    let checks = checks_upto(modules, upto);
    let has = |k: CheckKind| checks.iter().any(|(_, c)| c.kind == k);
    probe(env, &["--version"], "cargo", "rustup (https://rustup.rs)")?;
    if has(CheckKind::Fmt) {
        probe(
            env,
            &["fmt", "--version"],
            "rustfmt",
            "`rustup component add rustfmt`",
        )?;
    }
    if has(CheckKind::Clippy) {
        probe(
            env,
            &["clippy", "--version"],
            "clippy",
            "`rustup component add clippy`",
        )?;
    }

    let work = env.academy_dir.join(format!("work-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(io_err)?;
    let result = grade_in(&repo, &work, modules, &checks, env, cancel);
    let _ = fs::remove_dir_all(&work);
    result
}

fn grade_in(
    repo: &Repo,
    work: &Path,
    modules: &[Module],
    checks: &[(u8, Check)],
    env: &GradeEnv,
    cancel: &AtomicBool,
) -> Result<Report, GradeError> {
    let dest = work.join("project");
    let commit = git::export_head(repo.root(), &dest, &work.join("index"))
        .map_err(|e| GradeError::Git(e.to_string()))?;
    if !dest.join("Cargo.toml").is_file() {
        return Err(GradeError::NotCommitted);
    }
    let g = Grader { env, dest, cancel };
    let mut outcomes: HashMap<String, Outcome> = HashMap::new();
    let mut built = true;
    for (_, c) in checks {
        let r = match c.kind {
            CheckKind::Build => g.build()?,
            CheckKind::Fmt => g.fmt()?,
            CheckKind::Clippy => g.clippy()?,
            CheckKind::Test(_) => continue,
        };
        if c.kind == CheckKind::Build {
            built = r.is_ok();
        }
        outcomes.insert(c.kind.name().to_string(), r);
    }
    let mut by_module: Vec<(u8, Vec<&str>)> = Vec::new();
    for (m, c) in checks {
        if let CheckKind::Test(n) = c.kind {
            match by_module.iter_mut().find(|(k, _)| k == m) {
                Some((_, v)) => v.push(n),
                None => by_module.push((*m, vec![n])),
            }
        }
    }
    let tests_dir = g.dest.join("tests");
    fs::create_dir_all(&tests_dir).map_err(io_err)?;
    for (m, _) in &by_module {
        let src = modules
            .get(usize::from(*m) - 1)
            .map(|x| x.hidden_tests)
            .unwrap_or_default();
        fs::write(tests_dir.join(format!("academy_m{m:02}.rs")), src).map_err(io_err)?;
    }
    for (m, names) in &by_module {
        if built {
            outcomes.extend(g.tests(*m, names)?);
        } else {
            outcomes.extend(
                names
                    .iter()
                    .map(|n| (n.to_string(), Err(NO_BUILD.to_string()))),
            );
        }
    }
    let results = checks
        .iter()
        .map(|(m, c)| {
            let o = outcomes
                .get(c.kind.name())
                .cloned()
                .unwrap_or_else(|| Err("not run".into()));
            CheckResult {
                module: *m,
                name: c.kind.name().to_string(),
                passed: o.is_ok(),
                message: o.err().map(|e| clean(&e)).unwrap_or_default(),
                tags: c.tags.to_vec(),
            }
        })
        .collect();
    Ok(Report { commit, results })
}

/// A grading run on a worker thread.
pub struct Job {
    pub module: u8,
    pub rx: mpsc::Receiver<Result<Report, GradeError>>,
    pub cancel: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Job {
    /// Ask the run to stop and wait for the worker to finish.
    pub fn cancel_and_join(mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Start grading modules `1..=upto` of the shipped curriculum in the background.
pub fn spawn(project: PathBuf, upto: u8, env: GradeEnv) -> Job {
    let (tx, rx) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);
    let handle = thread::spawn(move || {
        let r = grade(&project, MODULES, usize::from(upto), &env, &flag);
        let _ = tx.send(r);
    });
    Job {
        module: upto,
        rx,
        cancel,
        handle: Some(handle),
    }
}
