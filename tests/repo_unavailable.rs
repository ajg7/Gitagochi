//! Tester pass: the "repo unavailable" warning and baseline retention.
//! Spec: a repo seen for the first time only baselines its HEAD; a configured
//! repo that cannot be read is warned about while others still feed.
//! Temp dirs only.

use gitagochi::pet::Pet;
use gitagochi::{repos, sync_all};
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=me@x.y",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

fn init_empty(p: &Path) {
    fs::create_dir_all(p).unwrap();
    git(p, &["init", "-q", "-b", "main"]);
}

fn init(p: &Path) {
    init_empty(p);
    commit(p, "base");
}

fn commit(p: &Path, msg: &str) {
    git(p, &["commit", "-q", "--allow-empty", "-m", msg]);
}

fn unavailable(errors: &[String]) -> usize {
    errors.iter().filter(|e| e.contains("unavailable")).count()
}

const ME: &str = "me@x.y";

#[test]
fn warning_appears_for_repo_with_baseline_whose_head_is_unreadable() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let rs = repos::resolve(&[a.clone(), b.clone()]).repos;
    let mut pet = Pet::new(0);
    let r0 = sync_all(&mut pet, &rs, ME, 0);
    assert!(r0.errors.is_empty(), "{:?}", r0.errors);

    fs::rename(&b, d.path().join("b-hidden")).unwrap();
    commit(&a, "a1");
    let r = sync_all(&mut pet, &rs, ME, 1);
    assert_eq!(unavailable(&r.errors), 1, "{:?}", r.errors);
    assert!(
        r.errors.iter().any(|e| e.starts_with("b:")),
        "{:?}",
        r.errors
    );
    assert_eq!(r.fed, 1, "the healthy repo still feeds");
}

#[test]
fn no_warning_for_empty_repo() {
    let d = tempfile::tempdir().unwrap();
    let e = d.path().join("empty");
    init_empty(&e);
    let rs = repos::resolve(std::slice::from_ref(&e)).repos;
    assert_eq!(rs.len(), 1);
    let mut pet = Pet::new(0);
    for t in 0..3 {
        let r = sync_all(&mut pet, &rs, ME, t);
        assert_eq!(unavailable(&r.errors), 0, "{:?}", r.errors);
        assert_eq!(r.fed, 0);
    }
}

#[test]
fn no_warning_for_freshly_added_repo() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let mut pet = Pet::new(0);
    let first = repos::resolve(std::slice::from_ref(&a)).repos;
    sync_all(&mut pet, &first, ME, 0);
    // b is added later (e.g. by a rescan) and syncs for the first time.
    let both = repos::resolve(&[a, b]).repos;
    let r = sync_all(&mut pet, &both, ME, 1);
    assert_eq!(unavailable(&r.errors), 0, "{:?}", r.errors);
    assert_eq!(r.fed, 0, "first sight only baselines");
}

#[test]
fn recovery_does_not_refeed_and_new_commits_feed() {
    let d = tempfile::tempdir().unwrap();
    let b = d.path().join("b");
    init(&b);
    for i in 0..5 {
        commit(&b, &format!("old{i}"));
    }
    let rs = repos::resolve(std::slice::from_ref(&b)).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, ME, 0);
    let fed_before = pet.total_fed;

    let hidden = d.path().join("hidden");
    fs::rename(&b, &hidden).unwrap();
    // Stays unavailable across several polls without feeding.
    for t in 1..4 {
        let r = sync_all(&mut pet, &rs, ME, t);
        assert_eq!(unavailable(&r.errors), 1);
        assert_eq!(r.fed, 0);
    }
    fs::rename(&hidden, &b).unwrap();

    let r = sync_all(&mut pet, &rs, ME, 10);
    assert_eq!(r.fed, 0, "recovery must not refeed history");
    assert_eq!(unavailable(&r.errors), 0, "warning clears: {:?}", r.errors);
    assert_eq!(pet.total_fed, fed_before);

    commit(&b, "after");
    commit(&b, "after2");
    assert_eq!(sync_all(&mut pet, &rs, ME, 11).fed, 2);
}

#[test]
fn commits_made_while_unavailable_feed_once_on_recovery() {
    let d = tempfile::tempdir().unwrap();
    let b = d.path().join("b");
    init(&b);
    let rs = repos::resolve(std::slice::from_ref(&b)).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, ME, 0);
    let hidden = d.path().join("hidden");
    fs::rename(&b, &hidden).unwrap();
    sync_all(&mut pet, &rs, ME, 1);
    commit(&hidden, "while-away");
    fs::rename(&hidden, &b).unwrap();
    assert_eq!(sync_all(&mut pet, &rs, ME, 2).fed, 1);
    assert_eq!(sync_all(&mut pet, &rs, ME, 3).fed, 0);
}

#[test]
fn baseline_survives_save_load_while_unavailable() {
    let d = tempfile::tempdir().unwrap();
    let b = d.path().join("b");
    init(&b);
    commit(&b, "old");
    let rs = repos::resolve(std::slice::from_ref(&b)).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, ME, 0);
    let hidden = d.path().join("hidden");
    fs::rename(&b, &hidden).unwrap();
    sync_all(&mut pet, &rs, ME, 1);

    let save = d.path().join("save.json");
    gitagochi::store::save(&save, &pet).unwrap();
    let mut pet2 = gitagochi::store::load(&save, 2);
    fs::rename(&hidden, &b).unwrap();
    assert_eq!(sync_all(&mut pet2, &rs, ME, 3).fed, 0);
    commit(&b, "new");
    assert_eq!(sync_all(&mut pet2, &rs, ME, 4).fed, 1);
}
