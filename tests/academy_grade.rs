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
