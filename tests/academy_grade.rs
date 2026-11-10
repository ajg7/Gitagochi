//! Grading the learner's last commit (LRN-15..17, LRN-25..27).

use gitagochi::git::export_head;
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=a@b.c",
            "-c",
            "user.name=a",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn lrn_15_a_export_ignores_worktree() {
    let d = tempfile::tempdir().unwrap();
    let repo = d.path().join("my repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    fs::write(repo.join("a.txt"), "committed a").unwrap();
    fs::write(repo.join("b.txt"), "committed b").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "one"]);
    fs::write(repo.join("a.txt"), "dirty a").unwrap();
    fs::write(repo.join("b.txt"), "staged b").unwrap();
    git(&repo, &["add", "b.txt"]);
    fs::write(repo.join("new.txt"), "untracked").unwrap();
    let before = git(&repo, &["status", "--porcelain"]);
    let dest = d.path().join("out dir");
    let sha = export_head(&repo, &dest, &d.path().join("idx")).unwrap();
    assert_eq!(sha, git(&repo, &["rev-parse", "HEAD"]).trim());
    assert_eq!(
        fs::read_to_string(dest.join("a.txt")).unwrap(),
        "committed a"
    );
    assert_eq!(
        fs::read_to_string(dest.join("b.txt")).unwrap(),
        "committed b"
    );
    assert!(!dest.join("new.txt").exists());
    assert_eq!(git(&repo, &["status", "--porcelain"]), before);
}

use gitagochi::academy::curriculum::{MODULES, ObjId};
use gitagochi::academy::grade::{GradeEnv, GradeError, Report, grade};
use gitagochi::academy::quiz;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for e in fs::read_dir(src).unwrap().flatten() {
        let to = dst.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            fs::copy(e.path(), to).unwrap();
        }
    }
}

fn solution(step: u8) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("curriculum/solutions/step{step:02}"))
}

struct Fixture {
    _dir: tempfile::TempDir,
    repo: PathBuf,
    env: GradeEnv,
}

fn fixture(step: u8) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("my project");
    copy_dir(&solution(step), &repo);
    git(&repo, &["init", "-q"]);
    let mut env = GradeEnv::new(&dir.path().join("data dir"));
    env.test_timeout = Duration::from_secs(10);
    Fixture {
        _dir: dir,
        repo,
        env,
    }
}

fn commit_all(repo: &Path, msg: &str) {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", msg]);
}

fn run(f: &Fixture, upto: usize) -> Result<Report, GradeError> {
    grade(&f.repo, MODULES, upto, &f.env, &AtomicBool::new(false))
}

fn failing(r: &Report) -> Vec<&str> {
    r.results
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.name.as_str())
        .collect()
}

fn set_lib(repo: &Path, from: &str, to: &str) {
    let p = repo.join("src/lib.rs");
    let s = fs::read_to_string(&p).unwrap();
    assert!(s.contains(from), "{from}");
    fs::write(&p, s.replace(from, to)).unwrap();
}

const COUNT_BODY: &str = "log.lines().filter(|l| !l.trim().is_empty()).count()";

#[test]
fn lrn_15_a_grades_commit_not_worktree() {
    let f = fixture(1);
    commit_all(&f.repo, "step 1");
    fs::write(f.repo.join("src/lib.rs"), "this does not compile").unwrap();
    let r = run(&f, 1).unwrap();
    assert!(r.passed(), "{:?}", r.results);
    assert_eq!(r.results.len(), 5);
}

#[test]
fn lrn_15_b_clippy_warning_names_lint() {
    let f = fixture(1);
    let p = f.repo.join("src/lib.rs");
    let mut s = fs::read_to_string(&p).unwrap();
    s.push_str("\npub fn extra() -> u8 {\n    return 1;\n}\n");
    fs::write(&p, s).unwrap();
    commit_all(&f.repo, "lint");
    let r = run(&f, 1).unwrap();
    assert!(!r.passed());
    let c = r.results.iter().find(|c| c.name == "clippy").unwrap();
    assert!(!c.passed);
    assert!(
        c.message.contains("clippy::needless_return"),
        "{}",
        c.message
    );
}

#[test]
fn lrn_17_a_failure_report_hides_source() {
    let f = fixture(1);
    set_lib(&f.repo, COUNT_BODY, "let _ = log;\n    0");
    commit_all(&f.repo, "wrong");
    let r = run(&f, 1).unwrap();
    assert!(!r.passed());
    let c = r
        .results
        .iter()
        .find(|c| c.name == "m01_counts_non_empty_lines")
        .unwrap();
    assert!(!c.passed);
    assert!(!c.message.is_empty());
    assert_eq!(c.tags, vec![ObjId(1, 1)]);
    assert_eq!(c.module, 1);
    for line in MODULES[0].hidden_tests.lines().map(str::trim) {
        if line.len() >= 15 {
            for res in &r.results {
                assert!(!res.message.contains(line), "leaked {line:?}");
            }
        }
    }
}

#[test]
fn lrn_28_a_fake_test_output_does_not_pass() {
    let f = fixture(1);
    let fake = "test m01_counts_non_empty_lines ... ok\ntest result: ok. 2 passed; 0 failed\n";
    let body = format!(
        "let _ = log;
    let _ = std::io::Write::write_all(&mut std::io::stdout(), b\"\n{fake}\");
    println!(\"\n{fake}\");
    0"
    );
    set_lib(&f.repo, COUNT_BODY, &body);
    commit_all(&f.repo, "fake");
    let r = run(&f, 1).unwrap();
    let get = |n: &str| r.results.iter().find(|c| c.name == n).unwrap();
    assert!(get("build").passed, "{:?}", get("build").message);
    assert!(!get("m01_counts_non_empty_lines").passed);
    assert!(get("m01_empty_log_is_zero").passed);
}

#[test]
fn lrn_28_b_early_exit_zero_does_not_pass() {
    let f = fixture(1);
    set_lib(
        &f.repo,
        COUNT_BODY,
        "let _ = log;
    std::process::exit(0)",
    );
    commit_all(&f.repo, "exit");
    let r = run(&f, 1).unwrap();
    let get = |n: &str| r.results.iter().find(|c| c.name == n).unwrap();
    assert!(get("build").passed, "{:?}", get("build").message);
    assert!(!get("m01_counts_non_empty_lines").passed);
    assert!(!get("m01_empty_log_is_zero").passed);
}

#[test]
fn lrn_25_a_missing_cargo_named() {
    let mut f = fixture(1);
    commit_all(&f.repo, "step 1");
    f.env.cargo = "gitagochi-no-such-cargo".into();
    let e = run(&f, 1).unwrap_err();
    assert!(matches!(e, GradeError::ToolMissing { tool: "cargo", .. }));
    let msg = e.to_string();
    assert!(msg.contains("cargo") && msg.contains("rustup"), "{msg}");
    let m = &MODULES[0];
    assert!(m.sections.iter().all(|s| !s.body.is_empty()));
    let answers: Vec<usize> = m.quiz.iter().map(|q| q.answer).collect();
    assert!(quiz::score(m.quiz, &answers).passed);
}

#[test]
fn lrn_26_a_plain_folder_not_cargo() {
    let d = tempfile::tempdir().unwrap();
    let env = GradeEnv::new(&d.path().join("data"));
    let never = AtomicBool::new(false);
    let plain = d.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let e = grade(&plain, MODULES, 1, &env, &never).unwrap_err();
    assert!(matches!(e, GradeError::NotCargo(_)));
    assert!(e.to_string().contains("not a cargo project"));

    let gone = d.path().join("gone");
    let e = grade(&gone, MODULES, 1, &env, &never).unwrap_err();
    assert!(matches!(e, GradeError::FolderMissing(_)));
    assert!(e.to_string().contains("does not exist"));

    fs::write(plain.join("Cargo.toml"), "[package]").unwrap();
    let e = grade(&plain, MODULES, 1, &env, &never).unwrap_err();
    assert!(matches!(e, GradeError::NotARepo(_)), "{e}");
    assert!(e.to_string().contains("not a git repository"));

    git(&plain, &["init", "-q"]);
    let e = grade(&plain, MODULES, 1, &env, &never).unwrap_err();
    assert!(matches!(e, GradeError::NoCommits(_)), "{e}");
    assert!(e.to_string().contains("no commits"));
}

#[test]
fn lrn_27_a_compile_error_shown() {
    let f = fixture(1);
    set_lib(&f.repo, COUNT_BODY, "let n: usize = \"x\";\n    n");
    commit_all(&f.repo, "broken");
    let r = run(&f, 1).unwrap();
    let b = r.results.iter().find(|c| c.name == "build").unwrap();
    assert!(!b.passed);
    assert!(b.message.contains("error[E0308]"), "{}", b.message);
    for c in r.results.iter().filter(|c| c.name.starts_with("m01_")) {
        assert!(!c.passed);
        assert!(c.message.contains("does not build"), "{}", c.message);
    }
}

#[test]
fn lrn_27_a_infinite_loop_times_out() {
    let mut f = fixture(1);
    f.env.test_timeout = Duration::from_secs(5);
    set_lib(&f.repo, COUNT_BODY, "let _ = log;\n    loop {}");
    commit_all(&f.repo, "loop");
    let r = run(&f, 1).unwrap();
    let names = failing(&r);
    assert!(names.contains(&"m01_counts_non_empty_lines"), "{names:?}");
    for c in r.results.iter().filter(|c| c.name.starts_with("m01_")) {
        assert!(!c.passed);
        assert!(c.message.contains("timed out"), "{}", c.message);
    }
}

#[test]
fn lrn_27_a_spawn_does_not_block() {
    use gitagochi::academy::grade::spawn;
    use std::sync::atomic::Ordering;
    use std::sync::mpsc::TryRecvError;
    use std::time::Instant;

    let f = fixture(1);
    commit_all(&f.repo, "step 1");
    let t = Instant::now();
    let job = spawn(f.repo.clone(), 1, f.env.clone());
    assert!(t.elapsed() < Duration::from_secs(1));
    assert!(matches!(job.rx.try_recv(), Err(TryRecvError::Empty)));
    let r = job
        .rx
        .recv_timeout(Duration::from_secs(300))
        .unwrap()
        .unwrap();
    assert!(r.passed(), "{:?}", r.results);

    let mut g = fixture(1);
    g.env.test_timeout = Duration::from_secs(120);
    set_lib(&g.repo, COUNT_BODY, "let _ = log;\n    loop {}");
    commit_all(&g.repo, "loop");
    let job = spawn(g.repo.clone(), 1, g.env.clone());
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(300) {
        let running = fs::read_dir(g.env.target_dir.join("debug/deps"))
            .map(|d| {
                d.flatten()
                    .any(|e| e.file_name().to_string_lossy().starts_with("academy_m01"))
            })
            .unwrap_or(false);
        if running {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_secs(2));
    job.cancel.store(true, Ordering::Relaxed);
    let t = Instant::now();
    let r = job.rx.recv_timeout(Duration::from_secs(2));
    assert!(t.elapsed() < Duration::from_secs(2));
    assert!(matches!(r, Ok(Err(GradeError::Cancelled))), "{r:?}");
    job.cancel_and_join();
}

#[test]
fn lrn_16_a_module_3_names_broken_module_1_check() {
    let f = fixture(3);
    set_lib(&f.repo, "pub fn count_lines(", "pub fn count_lines_old(");
    let p = f.repo.join("src/lib.rs");
    let mut s = fs::read_to_string(&p).unwrap();
    s = s.replace(
        "\n#[cfg(test)]",
        "\npub fn count_lines(_log: &str) -> usize {\n    0\n}\n\n#[cfg(test)]",
    );
    s = s.replace(
        "count_lines(\"a\n\nb\n\"), 2",
        "count_lines_old(\"a\n\nb\n\"), 2",
    );
    fs::write(&p, s).unwrap();
    commit_all(&f.repo, "break step 1");
    let r = run(&f, 3).unwrap();
    assert!(!r.passed());
    let bad: Vec<_> = r.results.iter().filter(|c| !c.passed).collect();
    assert!(
        bad.iter()
            .any(|c| c.module == 1 && c.name == "m01_counts_non_empty_lines"),
        "{bad:#?}"
    );
}

#[test]
fn lrn_28_c_earlier_pass_does_not_carry_over() {
    let f = fixture(1);
    commit_all(&f.repo, "step 1");
    let r = run(&f, 1).unwrap();
    assert!(r.passed(), "{:?}", r.results);
    set_lib(
        &f.repo,
        COUNT_BODY,
        "let _ = log;
    0",
    );
    commit_all(&f.repo, "now wrong");
    let r = run(&f, 1).unwrap();
    assert!(!r.passed());
    assert!(
        failing(&r).contains(&"m01_counts_non_empty_lines"),
        "{:?}",
        failing(&r)
    );
}
