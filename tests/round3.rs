//! Round-3 behaviour: persisted fed_ids dedup, vanished baseline, control
//! character stripping, old saves without fed_ids.

use gitagochi::git::Repo;
use gitagochi::pet::{MAX_FED_IDS, MAX_RECENT, Pet, RepoTrack};
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

fn commit(p: &Path, msg: &str) {
    git(p, &["commit", "-q", "--allow-empty", "-m", msg]);
}

#[test]
fn branch_switch_dedup_survives_save_load_beyond_recent_cap() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("repo");
    fs::create_dir(&p).unwrap();
    init(&p);
    let save = d.path().join("pet.json");
    let repo = Repo::open(&p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();

    git(&p, &["checkout", "-q", "-b", "feat"]);
    let n = MAX_RECENT + 10;
    for i in 0..n {
        commit(&p, &format!("f{i}"));
    }
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), n);
    git(&p, &["checkout", "-q", "main"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 0);

    // App restart between the switches.
    store::save(&save, &pet).unwrap();
    let mut pet = store::load(&save, 3);
    assert!(pet.repos.values().next().unwrap().fed_ids.len() >= n);

    git(&p, &["checkout", "-q", "feat"]);
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 3).unwrap(), 0);
    assert_eq!(pet.total_fed, n as u64);
}

#[test]
fn new_commit_after_switching_back_feeds_only_the_new_one() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    git(p, &["checkout", "-q", "-b", "feat"]);
    for i in 0..25 {
        commit(p, &format!("f{i}"));
    }
    sync(&mut pet, &repo, "a@b.c", 1).unwrap();
    git(p, &["checkout", "-q", "main"]);
    sync(&mut pet, &repo, "a@b.c", 2).unwrap();
    git(p, &["checkout", "-q", "feat"]);
    commit(p, "fresh");
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 3).unwrap(), 1);
    assert_eq!(pet.total_fed, 26);
}

#[test]
fn fed_ids_are_bounded_and_keep_newest() {
    let mut t = RepoTrack::default();
    let total = MAX_FED_IDS + 50;
    t.record((0..total).map(|i| format!("id{i}")));
    assert_eq!(t.fed_ids.len(), MAX_FED_IDS);
    assert!(t.has_fed(&format!("id{}", total - 1)));
    assert!(!t.has_fed("id0"));
}

#[test]
fn bloated_save_fed_ids_trimmed_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pet.json");
    let ids: Vec<String> = (0..MAX_FED_IDS + 500)
        .map(|i| format!("\"x{i}\""))
        .collect();
    fs::write(
        &path,
        format!(
            "{{\"repos\":{{\"k\":{{\"fed_ids\":[{}]}}}}}}",
            ids.join(",")
        ),
    )
    .unwrap();
    let pet = store::load(&path, 5);
    assert_eq!(pet.repos["k"].fed_ids.len(), MAX_FED_IDS);
    assert!(!dir.path().join("pet.json.bak").exists());
}

#[test]
fn old_minimal_save_loads_with_no_repos() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pet.json");
    fs::write(&path, r#"{"hunger":10,"health":50,"last_update":3}"#).unwrap();
    let pet = store::load(&path, 99);
    assert!(pet.repos.is_empty());
    assert_eq!(pet.last_update, 3);
    assert!(!dir.path().join("pet.json.bak").exists());
}

#[test]
fn fed_ids_round_trip_through_save() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("repo");
    fs::create_dir(&p).unwrap();
    init(&p);
    let repo = Repo::open(&p).unwrap();
    let path = dir.path().join("pet.json");
    let mut pet = Pet::new(1);
    sync(&mut pet, &repo, "a@b.c", 1).unwrap();
    for i in 0..30 {
        commit(&p, &format!("c{i}"));
    }
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 30);
    store::save(&path, &pet).unwrap();
    let back = store::load(&path, 2);
    assert_eq!(back.repos, pet.repos);
    assert_eq!(back.repos[repo.key()].fed_ids.len(), 30);
}

#[test]
fn garbage_baseline_rebaselines_without_feeding_or_error() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    pet.repos.values_mut().next().unwrap().last_seen = Some("not-a-commit-id".into());
    commit(p, "next");
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 0);
    assert_eq!(pet.repos.values().next().unwrap().last_seen, repo.head());
    commit(p, "after");
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 1);
}

#[test]
fn pruned_baseline_object_rebaselines_without_feeding() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    git(p, &["checkout", "-q", "-b", "tmp"]);
    commit(p, "doomed");
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 1);
    git(p, &["checkout", "-q", "main"]);
    git(p, &["branch", "-q", "-D", "tmp"]);
    git(p, &["reflog", "expire", "--expire=now", "--all"]);
    git(p, &["gc", "-q", "--prune=now"]);
    commit(p, "later");
    let fed_before = pet.total_fed;
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 2).unwrap(), 0);
    assert_eq!(pet.total_fed, fed_before);
    assert_eq!(pet.repos.values().next().unwrap().last_seen, repo.head());
    commit(p, "after");
    assert_eq!(sync(&mut pet, &repo, "a@b.c", 3).unwrap(), 1);
}

#[test]
fn ansi_and_control_characters_are_stripped_from_summaries() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("repo");
    fs::create_dir(&p).unwrap();
    init(&p);
    let repo = Repo::open(&p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();

    let msg = d.path().join("msg.txt");
    fs::write(
        &msg,
        "start \u{1b}[31mRED\u{1b}[0m mid \u{7}bell\u{8}\u{7f} \u{1b}[2J\u{1b}[1;1Hend\n",
    )
    .unwrap();
    let msg_s = msg.to_string_lossy().into_owned();
    git(
        &p,
        &[
            "commit",
            "-q",
            "--allow-empty",
            "--cleanup=verbatim",
            "-F",
            &msg_s,
        ],
    );

    assert_eq!(sync(&mut pet, &repo, "a@b.c", 1).unwrap(), 1);
    let s = &pet.recent[0].summary;
    assert!(!s.chars().any(char::is_control), "control char left: {s:?}");
    assert!(
        !s.contains("[31m") && !s.contains("[2J") && !s.contains("[1;1H"),
        "{s:?}"
    );
    assert!(s.contains("start") && s.contains("RED") && s.contains("mid") && s.contains("end"));
}

#[test]
fn plain_and_unicode_summaries_are_untouched() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    init(p);
    let repo = Repo::open(p).unwrap();
    let mut pet = Pet::new(0);
    sync(&mut pet, &repo, "a@b.c", 0).unwrap();
    let msg = "fix: caf\u{e9} \u{65e5}\u{672c}\u{8a9e} [wip] (#12)";
    commit(p, msg);
    sync(&mut pet, &repo, "a@b.c", 1).unwrap();
    assert_eq!(pet.recent[0].summary, msg);
}
