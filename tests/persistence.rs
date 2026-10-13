//! Save/load edge cases. All files live in temp dirs.

use gitagochi::pet::{FedCommit, Pet};
use gitagochi::store;
use std::fs;
use std::path::Path;

#[test]
fn missing_file_starts_fresh() {
    let d = tempfile::tempdir().unwrap();
    let p = store::load(&d.path().join("nope.json"), 77);
    assert_eq!(p, Pet::new(77));
}

#[test]
fn missing_parent_dir_is_created_on_save() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("a").join("b").join("pet.json");
    store::save(&f, &Pet::new(1)).unwrap();
    assert!(f.exists());
}

#[test]
fn corrupt_variants_start_fresh_and_keep_bak() {
    let cases = [
        "",
        "{ not json",
        "[]",
        "null",
        "{\"hunger\": \"lots\"}",
        "\u{0}\u{1}\u{2}",
    ];
    for (i, junk) in cases.iter().enumerate() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("pet.json");
        fs::write(&f, junk).unwrap();
        let p = store::load(&f, 9);
        assert_eq!(p, Pet::new(9), "case {i}");
        let bak = d.path().join("pet.json.bak");
        assert_eq!(fs::read_to_string(bak).unwrap(), *junk, "case {i}");
    }
}

#[test]
fn binary_garbage_does_not_crash() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("pet.json");
    fs::write(&f, [0xffu8, 0xfe, 0x00, 0x80]).unwrap();
    assert_eq!(store::load(&f, 3), Pet::new(3));
    assert!(d.path().join("pet.json.bak").exists());
}

#[test]
fn directory_in_place_of_file_does_not_crash() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("pet.json");
    fs::create_dir(&f).unwrap();
    let _ = store::load(&f, 3);
}

#[test]
fn out_of_range_values_are_clamped() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("pet.json");
    let mut p = Pet::new(1);
    p.hunger = 1e9;
    p.health = -50.0;
    store::save(&f, &p).unwrap();
    let l = store::load(&f, 2);
    assert!((0.0..=100.0).contains(&l.hunger));
    assert!((0.0..=100.0).contains(&l.health));
}

#[test]
fn round_trip_preserves_state() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("pet.json");
    let mut p = Pet::new(5);
    p.feed(&[FedCommit {
        id: "a".into(),
        summary: "unicode \u{2713}".into(),
        time: 3,
        repo: "r".into(),
    }]);
    let t = p.repos.entry("k".into()).or_default();
    t.initialized = true;
    t.last_seen = Some("a".into());
    t.fed_ids.push("a".into());
    store::save(&f, &p).unwrap();
    assert_eq!(store::load(&f, 100), p);
}

#[test]
fn legacy_save_path_per_repo_in_data_dir() {
    let d = Path::new("data");
    let a = store::save_path(d, Path::new("/x/repo"));
    let b = store::save_path(d, Path::new("/y/repo"));
    assert_ne!(a, b);
    assert_eq!(a.parent(), Some(d));
    assert_eq!(a, store::save_path(d, Path::new("/x/repo")));
}
