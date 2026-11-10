//! Project setup (LRN-12, LRN-13).

use gitagochi::academy::scaffold::{ScaffoldError, create};
use gitagochi::git::Repo;
use std::fs;
use std::path::Path;

fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| {
            let p = e.path();
            let data = if p.is_file() {
                fs::read(&p).unwrap()
            } else {
                vec![]
            };
            (e.file_name().to_string_lossy().into_owned(), data)
        })
        .collect();
    v.sort();
    v
}

#[test]
fn lrn_12_a_scaffold_is_cargo_git() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("my tracker");
    create(&dir).unwrap();
    for f in ["Cargo.toml", "src/lib.rs", "src/main.rs", ".gitignore"] {
        assert!(dir.join(f).is_file(), "{f}");
    }
    assert!(
        fs::read_to_string(dir.join("Cargo.toml"))
            .unwrap()
            .contains("name = \"tracker\"")
    );
    assert!(Repo::open(&dir).is_ok());
    let empty = d.path().join("empty");
    fs::create_dir(&empty).unwrap();
    create(&empty).unwrap();
    assert!(empty.join("Cargo.toml").is_file());
}

#[test]
fn lrn_13_a_non_empty_folder_refused_unchanged() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("busy");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("notes.txt"), "keep me").unwrap();
    let before = snapshot(&dir);
    assert!(matches!(create(&dir), Err(ScaffoldError::NotEmpty(_))));
    assert_eq!(snapshot(&dir), before);
    let file = d.path().join("a file");
    fs::write(&file, "x").unwrap();
    assert!(matches!(create(&file), Err(ScaffoldError::NotEmpty(_))));
    assert_eq!(fs::read_to_string(&file).unwrap(), "x");
}

use gitagochi::academy::progress::Progress;
use gitagochi::pet::Pet;
use gitagochi::repos::resolve;
use gitagochi::{sync_all, watched_paths};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=learner@x.io",
            "-c",
            "user.name=l",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn lrn_12_a_project_is_watched() {
    let d = tempfile::tempdir().unwrap();
    let project = d.path().join("my tracker");
    create(&project).unwrap();
    let other = d.path().join("other");
    fs::create_dir_all(&other).unwrap();
    git(&other, &["init", "-q"]);
    let progress = Progress {
        project: Some(project.clone()),
        ..Progress::default()
    };
    let key = Repo::open(&project).unwrap().key().to_string();
    for cli in [vec![], vec![other.clone()], vec![project.clone()]] {
        let r = resolve(&watched_paths(cli, &progress));
        assert!(r.repos.iter().any(|x| x.key() == key), "not watched");
        assert_eq!(r.repos.iter().filter(|x| x.key() == key).count(), 1);
    }
}

#[test]
fn lrn_12_b_project_commit_feeds_pet() {
    let d = tempfile::tempdir().unwrap();
    let project = d.path().join("my tracker");
    create(&project).unwrap();
    let progress = Progress {
        project: Some(project.clone()),
        ..Progress::default()
    };
    let repos = resolve(&watched_paths(vec![], &progress)).repos;
    let mut pet = Pet::new(0);
    assert_eq!(sync_all(&mut pet, &repos, "learner@x.io", 0).fed, 0);
    git(&project, &["add", "-A"]);
    git(&project, &["commit", "-q", "-m", "step 1"]);
    let r = sync_all(&mut pet, &repos, "learner@x.io", 1);
    assert_eq!(r.fed, 1, "{:?}", r.errors);
    assert_eq!(pet.total_fed, 1);
}
