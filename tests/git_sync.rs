use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::sync;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "git {args:?} failed");
}

fn commit(dir: &Path, email: &str, msg: &str) {
    let name = "user.name=T";
    let mail = format!("user.email={email}");
    git(
        dir,
        &[
            "-c",
            name,
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
fn feeds_only_new_commits_by_author() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);

    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);

    // Empty repo: baseline, nothing fed.
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 0).unwrap(), 0);

    // First commit in a previously empty repo is fed.
    commit(p, "me@x.com", "first");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 1);

    // Others' commits do not feed; own commits do; no double feeding.
    commit(p, "other@x.com", "theirs");
    commit(p, "me@x.com", "mine");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 2).unwrap(), 1);
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 3).unwrap(), 0);
    assert_eq!(pet.total_fed, 2);
    assert_eq!(pet.recent[0].summary, "mine");
}

#[test]
fn existing_history_is_baseline_only() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    git(p, &["init", "-q"]);
    commit(p, "me@x.com", "old");
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 0).unwrap(), 0);
    commit(p, "me@x.com", "new");
    assert_eq!(sync(&mut pet, &repo, "me@x.com", 1).unwrap(), 1);
}

#[test]
fn non_git_directory_is_a_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = Repo::open(dir.path()).err().expect("should fail");
    assert!(err.to_string().contains("not inside a git repository"));
}
