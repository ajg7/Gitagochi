//! Progress persistence (LRN-23, LRN-24).

use gitagochi::academy::progress::Progress;
use gitagochi::store::{load_progress, progress_path, save_progress};
use std::fs;
use std::path::PathBuf;

#[test]
fn lrn_23_a_restart_restores_progress() {
    let d = tempfile::tempdir().unwrap();
    let path = progress_path(d.path());
    let mut p = Progress::default();
    p.record_quiz(1, 90);
    p.record_project_pass(1);
    p.project = Some(PathBuf::from(r"C:\Users\me\my tracker"));
    save_progress(&path, &p).unwrap();
    let back = load_progress(&path, 10);
    assert!(back.passed(1));
    assert!(back.open(2));
    assert!(!back.open(3));
    assert_eq!(back.project, Some(PathBuf::from(r"C:\Users\me\my tracker")));
}

#[test]
fn lrn_24_a_corrupt_progress_backed_up() {
    let d = tempfile::tempdir().unwrap();
    let path = progress_path(d.path());
    let missing = load_progress(&path, 10);
    assert_eq!(missing, Progress::default());
    assert!(missing.open(1) && !missing.open(2));
    for bad in ["{ not json", "[1,2]"] {
        fs::write(&path, bad).unwrap();
        let p = load_progress(&path, 10);
        assert_eq!(p, Progress::default());
        assert!(p.open(1) && !p.open(2));
        assert!(!path.exists());
        assert_eq!(
            fs::read_to_string(d.path().join("progress.json.bak")).unwrap(),
            bad
        );
    }
}
