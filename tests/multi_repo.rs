//! Multi-repo behaviour: one shared pet fed by several real repos.

use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::repos::resolve;
use gitagochi::{store, sync_all};
use std::fs;
use std::path::{Path, PathBuf};
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
    fs::create_dir_all(p).unwrap();
    git(p, &["init", "-q", "-b", "main"]);
    git(p, &["commit", "-q", "--allow-empty", "-m", "base"]);
}

fn commit(p: &Path, msg: &str) {
    git(p, &["commit", "-q", "--allow-empty", "-m", msg]);
}

fn names(repos: &[Repo]) -> Vec<String> {
    let mut v: Vec<String> = repos.iter().map(|r| r.name().to_string()).collect();
    v.sort();
    v
}

#[test]
fn two_repos_feed_one_pet_with_independent_baselines() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let repos = resolve(&[a.clone(), b.clone()]).repos;
    assert_eq!(repos.len(), 2);
    let mut pet = Pet::new(0);
    assert_eq!(sync_all(&mut pet, &repos, "a@b.c", 0).fed, 0);
    commit(&a, "in a");
    commit(&b, "in b1");
    commit(&b, "in b2");
    let r = sync_all(&mut pet, &repos, "a@b.c", 1);
    assert_eq!(r.fed, 3);
    assert!(r.errors.is_empty());
    assert_eq!(pet.total_fed, 3);
    assert_eq!(pet.repos.len(), 2);
    let from: Vec<&str> = pet.recent.iter().map(|c| c.repo.as_str()).collect();
    assert!(from.contains(&"a") && from.contains(&"b"));
    assert_eq!(sync_all(&mut pet, &repos, "a@b.c", 2).fed, 0);
}

#[test]
fn added_repo_baselines_only_itself_and_removed_repo_resumes_without_refeed() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    commit(&b, "old history");
    let ra = resolve(std::slice::from_ref(&a)).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &ra, "a@b.c", 0);
    commit(&a, "a1");
    assert_eq!(sync_all(&mut pet, &ra, "a@b.c", 1).fed, 1);

    let both = resolve(&[a.clone(), b.clone()]).repos;
    commit(&a, "a2");
    assert_eq!(sync_all(&mut pet, &both, "a@b.c", 2).fed, 1);
    commit(&b, "b1");
    assert_eq!(sync_all(&mut pet, &both, "a@b.c", 3).fed, 1);

    commit(&b, "b2 while b unwatched");
    assert_eq!(sync_all(&mut pet, &ra, "a@b.c", 4).fed, 0);
    assert_eq!(sync_all(&mut pet, &both, "a@b.c", 5).fed, 1);
    assert_eq!(pet.total_fed, 4);
}

#[test]
fn save_is_shared_regardless_of_repo_selection() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let save = store::shared_path(&d.path().join("data"));
    let ra = resolve(std::slice::from_ref(&a)).repos;
    let mut pet = store::load(&save, 0);
    sync_all(&mut pet, &ra, "a@b.c", 0);
    commit(&a, "x");
    sync_all(&mut pet, &ra, "a@b.c", 1);
    store::save(&save, &pet).unwrap();

    let rb = resolve(std::slice::from_ref(&b)).repos;
    let mut pet = store::load(&save, 2);
    assert_eq!(pet.total_fed, 1);
    sync_all(&mut pet, &rb, "a@b.c", 2);
    commit(&b, "y");
    assert_eq!(sync_all(&mut pet, &rb, "a@b.c", 3).fed, 1);
    assert_eq!(pet.total_fed, 2);
}

#[test]
fn one_failing_repo_does_not_stop_others() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let repos = resolve(&[a.clone(), b.clone()]).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &repos, "a@b.c", 0);
    commit(&a, "x");
    fs::remove_dir_all(b.join(".git")).unwrap();
    let r = sync_all(&mut pet, &repos, "a@b.c", 1);
    assert_eq!(r.fed, 1);
}

#[test]
fn parent_folder_with_spaces_and_ampersand_scans_depths_1_to_3() {
    let d = tempfile::tempdir().unwrap();
    let parent = d.path().join("Tech & Coding");
    init(&parent.join("one"));
    init(&parent.join("Coding Project").join("two"));
    init(&parent.join("x").join("y").join("three"));
    init(&parent.join("x").join("y").join("z").join("too deep"));
    init(&parent.join("one").join("sub").join("nested"));
    init(&parent.join("node_modules").join("dep"));
    init(&parent.join("target").join("dep"));
    init(&parent.join(".hidden").join("dep"));
    fs::create_dir_all(parent.join("plain").join("dir")).unwrap();

    let r = resolve(std::slice::from_ref(&parent));
    assert_eq!(names(&r.repos), ["one", "three", "two"]);
    assert!(r.warnings.is_empty());

    let mut pet = Pet::new(0);
    sync_all(&mut pet, &r.repos, "a@b.c", 0);
    commit(&parent.join("Coding Project").join("two"), "hi");
    assert_eq!(sync_all(&mut pet, &r.repos, "a@b.c", 1).fed, 1);
}

#[test]
fn rescan_picks_up_new_repos() {
    let d = tempfile::tempdir().unwrap();
    init(&d.path().join("first"));
    let paths = [d.path().to_path_buf()];
    assert_eq!(resolve(&paths).repos.len(), 1);
    init(&d.path().join("deep").join("second"));
    assert_eq!(resolve(&paths).repos.len(), 2);
}

#[test]
fn missing_path_warns_but_others_resolve() {
    let d = tempfile::tempdir().unwrap();
    init(&d.path().join("a"));
    let r = resolve(&[d.path().join("nope"), d.path().join("a")]);
    assert_eq!(r.repos.len(), 1);
    assert_eq!(r.warnings.len(), 1);
    let none = resolve(&[d.path().join("nope")]);
    assert!(none.repos.is_empty());
}

#[test]
fn subdirectory_of_a_repo_still_resolves_to_the_repo() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    fs::create_dir(a.join("src")).unwrap();
    let r = resolve(&[a.join("src")]);
    assert_eq!(names(&r.repos), ["a"]);
}

fn write_legacy(data: &Path, repo: &Repo, body: &str) -> PathBuf {
    fs::create_dir_all(data).unwrap();
    let f = store::save_path(data, Path::new(repo.key()));
    fs::write(&f, body).unwrap();
    f
}

#[test]
fn legacy_save_is_adopted_with_tracking_and_does_not_refeed() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let repo = Repo::open(&a).unwrap();
    let head = repo.head().unwrap();
    let data = d.path().join("data");
    let body = format!(
        r#"{{"hunger":40.0,"health":90.0,"total_fed":7,"last_update":10,
"initialized":true,"last_seen":"{head}","fed_ids":["{head}"],
"recent":[{{"id":"{head}","summary":"base","time":1}}]}}"#
    );
    write_legacy(&data, &repo, &body);

    let mut pet = store::migrate_legacy(&data, &[repo.key().to_string()], 11).unwrap();
    assert_eq!(pet.total_fed, 7);
    assert!(pet.repos[repo.key()].initialized);
    let repos = vec![repo];
    assert_eq!(sync_all(&mut pet, &repos, "a@b.c", 12).fed, 0);
    commit(&a, "new");
    assert_eq!(sync_all(&mut pet, &repos, "a@b.c", 13).fed, 1);
    assert_eq!(pet.total_fed, 8);

    store::save(&store::shared_path(&data), &pet).unwrap();
    assert!(store::migrate_legacy(&data, &[], 14).is_none());
}

#[test]
fn newest_of_several_legacy_saves_wins() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let (ra, rb) = (Repo::open(&a).unwrap(), Repo::open(&b).unwrap());
    let data = d.path().join("data");
    write_legacy(&data, &ra, r#"{"total_fed":1,"last_update":1}"#);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    write_legacy(&data, &rb, r#"{"total_fed":2,"last_update":1}"#);
    let keys = [ra.key().to_string(), rb.key().to_string()];
    let pet = store::migrate_legacy(&data, &keys, 5).unwrap();
    assert_eq!(pet.total_fed, 2);
}

#[test]
fn no_legacy_saves_means_no_migration() {
    let d = tempfile::tempdir().unwrap();
    assert!(store::migrate_legacy(&d.path().join("missing"), &[], 0).is_none());
    assert!(store::migrate_legacy(d.path(), &[], 0).is_none());
}
