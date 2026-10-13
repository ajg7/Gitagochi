use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::sync;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let st = Command::new("git")
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
    assert!(
        st.status.success(),
        "{:?}",
        String::from_utf8_lossy(&st.stderr)
    );
}

#[test]
fn branch_switch_does_not_refeed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    git(p, &["init", "-q", "-b", "main"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "base"]);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 0).unwrap(), 0);

    git(p, &["checkout", "-q", "-b", "feat"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "f1"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 1);

    git(p, &["checkout", "-q", "main"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "m1"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 1);

    git(p, &["checkout", "-q", "feat"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 3).unwrap(), 0);
    assert_eq!(pet.total_fed, 2);
}

#[test]
fn non_git_dir_is_not_a_repo_error() {
    let d = tempfile::tempdir().unwrap();
    let e = Repo::open(d.path()).err().unwrap();
    assert!(e.to_string().contains("not inside a git repository"));
}

#[test]
fn vanished_baseline_rebaselines_without_feeding() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    git(p, &["init", "-q", "-b", "main"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "base"]);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    pet.last_seen = Some("0123456789012345678901234567890123456789".into());
    git(p, &["commit", "-q", "--allow-empty", "-m", "next"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 0);
    assert_eq!(pet.last_seen, repo.head());
    git(p, &["commit", "-q", "--allow-empty", "-m", "after"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 1);
}

#[test]
fn old_save_without_fed_ids_still_deduplicates_recent() {
    let mut pet: Pet =
        serde_json::from_str(r#"{"recent":[{"id":"abc","summary":"s","time":1}]}"#).unwrap();
    assert!(pet.fed_ids.is_empty());
    pet.sanitize();
    assert!(pet.has_fed("abc"));
}
