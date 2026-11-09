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
