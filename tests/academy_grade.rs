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
