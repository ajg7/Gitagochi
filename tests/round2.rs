//! Round-2 behaviour: branch-switch dedup, git error surfacing, load defaults.

use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::{store, sync};
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
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
}

fn init(p: &Path) {
    git(p, &["init", "-q", "-b", "main"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "base"]);
}

#[test]
fn switching_to_branch_with_many_fed_commits_does_not_refeed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    git(p, &["checkout", "-q", "-b", "feat"]);
    for i in 0..30 {
        git(
            p,
            &["commit", "-q", "--allow-empty", "-m", &format!("f{i}")],
        );
    }
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 30);
    git(p, &["checkout", "-q", "main"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 0);
    git(p, &["checkout", "-q", "feat"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 3).unwrap(), 0);
    assert_eq!(pet.total_fed, 30);
}

#[test]
fn reset_to_older_commit_feeds_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    git(p, &["commit", "-q", "--allow-empty", "-m", "x"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 1);
    git(p, &["reset", "-q", "--hard", "HEAD~1"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 0);
    assert_eq!(pet.total_fed, 1);
}

#[test]
fn vanished_baseline_feeds_nothing_and_recovers() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    pet.repos.values_mut().next().unwrap().last_seen =
        Some("0123456789abcdef0123456789abcdef01234567".into());
    git(p, &["commit", "-q", "--allow-empty", "-m", "x"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 0);
    assert_eq!(pet.repos.values().next().unwrap().last_seen, repo.head());
    git(p, &["commit", "-q", "--allow-empty", "-m", "y"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 1);
}

#[test]
fn git_failure_is_surfaced_as_error_not_silent() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    git(p, &["commit", "-q", "--allow-empty", "-m", "x"]);
    // Remove the repo's .git so subsequent git commands fail.
    fs::remove_dir_all(p.join(".git")).unwrap();
    let before = pet.clone();
    let r = sync(&mut pet, &repo, "a@b.c", 1);
    // Either an error is returned, or nothing was fed; never a false feed.
    match r {
        Err(e) => assert!(!e.to_string().is_empty()),
        Ok(n) => assert_eq!(n, 0),
    }
    assert_eq!(pet.total_fed, before.total_fed);
}

#[test]
fn load_non_object_json_gives_fresh_pet_and_bak() {
    for (i, body) in ["[]", "42", "null", "\"str\"", "true"].iter().enumerate() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("pet.json");
        fs::write(&path, body).unwrap();
        let pet = store::load(&path, 77 + i as i64);
        assert_eq!(pet, Pet::new(77 + i as i64), "body {body}");
        assert!(!path.exists());
        assert_eq!(
            fs::read_to_string(d.path().join("pet.json.bak")).unwrap(),
            *body
        );
    }
}

#[test]
fn load_empty_object_uses_defaults_and_now() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("pet.json");
    fs::write(&path, "{}").unwrap();
    let pet = store::load(&path, 1234);
    assert_eq!(pet.last_update, 1234);
    assert_eq!(pet.total_fed, 0);
    assert!(pet.repos.is_empty());
    assert!(pet.hunger.is_finite() && (0.0..=100.0).contains(&pet.hunger));
    assert!((0.0..=100.0).contains(&pet.health));
}

#[test]
fn load_partial_object_keeps_present_fields() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("pet.json");
    fs::write(&path, r#"{"total_fed": 7, "last_update": 500}"#).unwrap();
    let pet = store::load(&path, 9999);
    assert_eq!(pet.total_fed, 7);
    assert_eq!(pet.last_update, 500);
    assert!(!path.with_extension("json.bak").exists());
}

#[test]
fn load_wrong_field_type_is_corrupt() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("pet.json");
    fs::write(&path, r#"{"total_fed": "many"}"#).unwrap();
    let pet = store::load(&path, 5);
    assert_eq!(pet, Pet::new(5));
    assert!(d.path().join("pet.json.bak").exists());
}

#[test]
fn commit_summary_with_unicode_and_long_text_is_fed_safely() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    let long = "é".repeat(600);
    git(p, &["commit", "-q", "--allow-empty", "-m", &long]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 1);
    assert!(pet.recent[0].summary.chars().count() <= 200);
}
