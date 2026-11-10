//! Reference solutions pass their module and the previous step fails it (LRN-04.a, LRN-16).

use gitagochi::academy::curriculum::MODULES;
use gitagochi::academy::grade::{GradeEnv, Report, grade};
use gitagochi::academy::scaffold;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

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

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for e in fs::read_dir(src).unwrap().flatten() {
        let to = dst.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            fs::copy(e.path(), to).unwrap();
        }
    }
}

fn graded(root: &Path, step: usize, upto: usize) -> Report {
    let repo = root.join(format!("step {step}"));
    if step == 0 {
        scaffold::create(&repo).unwrap();
    } else {
        let src = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("curriculum/solutions/step{step:02}"));
        copy_dir(&src, &repo);
        git(&repo, &["init", "-q"]);
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "step"]);
    let mut env = GradeEnv::new(&root.join("data"));
    env.test_timeout = Duration::from_secs(60);
    grade(&repo, MODULES, upto, &env, &AtomicBool::new(false)).unwrap()
}

#[test]
fn lrn_04_a_reference_solutions() {
    std::thread::scope(|s| {
        for n in 1..=MODULES.len() {
            s.spawn(move || {
                let d = tempfile::tempdir().unwrap();
                let r = graded(&d.path().join("pass"), n, n);
                let bad: Vec<_> = r.results.iter().filter(|c| !c.passed).collect();
                assert!(bad.is_empty(), "step {n} fails module {n}: {bad:#?}");
                let r = graded(&d.path().join("prev"), n - 1, n);
                assert!(
                    r.results
                        .iter()
                        .any(|c| !c.passed && usize::from(c.module) == n),
                    "step {} passes module {n}'s own checks",
                    n - 1
                );
            });
        }
    });
}
