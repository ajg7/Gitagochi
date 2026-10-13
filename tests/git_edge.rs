//! Git ingestion edge cases, using throwaway repos in temp dirs.

use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::sync;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?} failed");
}

fn init() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "-q"]);
    d
}

fn commit(dir: &Path, email: &str, msg: &str) {
    let mail = format!("user.email={email}");
    git(
        dir,
        &[
            "-c",
            "user.name=T",
            "-c",
            &mail,
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            msg,
        ],
    );
}

#[test]
fn non_git_directory_gives_clear_error() {
    let d = tempfile::tempdir().unwrap();
    let e = Repo::open(d.path()).err().expect("must fail");
    let m = e.to_string();
    assert!(m.contains("not inside a git repository"), "{m}");
}

#[test]
fn nonexistent_path_is_an_error_not_a_panic() {
    let d = tempfile::tempdir().unwrap();
    assert!(Repo::open(&d.path().join("missing")).is_err());
}

#[test]
fn subdirectory_of_repo_opens() {
    let d = init();
    let sub = d.path().join("a");
    std::fs::create_dir(&sub).unwrap();
    assert!(Repo::open(&sub).is_ok());
}

#[test]
fn empty_repo_syncs_without_error_and_feeds_nothing() {
    let d = init();
    let repo = Repo::open(d.path()).unwrap();
    assert!(repo.head().is_none());
    let mut pet = Pet::new(0);
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 0).unwrap(), 0);
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 10).unwrap(), 0);
    assert_eq!(pet.total_fed, 0);
}

#[test]
fn only_configured_author_feeds_and_case_is_ignored() {
    let d = init();
    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "me@x.com", 0).unwrap();
    commit(d.path(), "other@x.com", "o1");
    commit(d.path(), "ME@X.COM", "m1");
    commit(d.path(), "me@x.com.evil", "partial");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 1);
    assert_eq!(pet.recent[0].summary, "m1");
}

#[test]
fn author_with_regex_chars_is_literal() {
    let d = init();
    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a+b@x.com", 0).unwrap();
    commit(d.path(), "aab@x.com", "no");
    commit(d.path(), "a+b@x.com", "yes");
    assert_eq!(sync(&mut pet, &repo, "a+b@x.com", 1).unwrap(), 1);
}

#[test]
fn default_author_reads_git_config() {
    let d = init();
    git(d.path(), &["config", "user.email", "cfg@x.com"]);
    let repo = Repo::open(d.path()).unwrap();
    assert_eq!(repo.default_author().as_deref(), Some("cfg@x.com"));
}

#[test]
fn no_double_feeding_across_restart() {
    let d = init();
    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "me@x.com", 0).unwrap();
    commit(d.path(), "me@x.com", "a");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 1);
    // simulate restart through serialization
    let json = serde_json::to_string(&pet).unwrap();
    let mut pet2: Pet = serde_json::from_str(&json).unwrap();
    let repo2 = Repo::open(d.path()).unwrap();
    assert_eq!(sync(&mut pet2, &repo2, "me@x.com", 2).unwrap(), 0);
    commit(d.path(), "me@x.com", "b");
    assert_eq!(sync(&mut pet2, &repo2, "me@x.com", 3).unwrap(), 1);
    assert_eq!(pet2.total_fed, 2);
}

#[test]
fn commits_made_while_app_closed_feed_on_next_start() {
    let d = init();
    commit(d.path(), "me@x.com", "old");
    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "me@x.com", 0).unwrap();
    commit(d.path(), "me@x.com", "while away 1");
    commit(d.path(), "me@x.com", "while away 2");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 100).unwrap(), 2);
}

#[test]
fn rewritten_history_does_not_error_or_feed() {
    let d = init();
    commit(d.path(), "me@x.com", "one");
    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "me@x.com", 0).unwrap();
    pet.last_seen = Some("0123456789012345678901234567890123456789".into());
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 0);
    assert_eq!(pet.last_seen, repo.head());
}

#[test]
fn very_large_history_only_new_commits_are_walked() {
    let d = init();
    // Build 5000 commits quickly with fast-import.
    let mut stream = String::new();
    for i in 0..5000 {
        stream.push_str(&format!(
            "commit refs/heads/master\ncommitter T <me@x.com> {} +0000\ndata 2\nc{}\n\n",
            1_600_000_000 + i,
            i % 10
        ));
    }
    let mut child = Command::new("git")
        .arg("-C")
        .arg(d.path())
        .args(["fast-import", "--quiet"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    git(d.path(), &["symbolic-ref", "HEAD", "refs/heads/master"]);

    let repo = Repo::open(d.path()).unwrap();
    let mut pet = Pet::new(0);
    let start = std::time::Instant::now();
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 0).unwrap(), 0);
    assert_eq!(pet.total_fed, 0, "existing history must not feed");
    commit(d.path(), "me@x.com", "new");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 1);
    assert_eq!(pet.total_fed, 1);
    assert!(start.elapsed().as_secs() < 30, "took {:?}", start.elapsed());
}
