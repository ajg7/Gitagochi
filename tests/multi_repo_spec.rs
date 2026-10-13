//! Tester pass for "Multi-repo (v1.1)" in docs/spec.md. Temp dirs only.

use gitagochi::git::Repo;
use gitagochi::pet::Pet;
use gitagochi::repos::{self, Config};
use gitagochi::{effective_paths, parse_args, store, sync_all};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git_as(dir: &Path, email: &str, args: &[&str]) {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            &format!("user.email={email}"),
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

fn init(p: &Path) {
    fs::create_dir_all(p).unwrap();
    git_as(p, "me@x.y", &["init", "-q", "-b", "main"]);
    git_as(
        p,
        "me@x.y",
        &["commit", "-q", "--allow-empty", "-m", "base"],
    );
}

fn commit(p: &Path, msg: &str) {
    commit_as(p, "me@x.y", msg);
}

fn commit_as(p: &Path, email: &str, msg: &str) {
    git_as(p, email, &["commit", "-q", "--allow-empty", "-m", msg]);
}

fn names(r: &repos::Resolved) -> Vec<String> {
    let mut v: Vec<String> = r.repos.iter().map(|r| r.name().to_string()).collect();
    v.sort();
    v
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

// ---- selection precedence ----

#[test]
fn cli_paths_override_config_repos() {
    let cfg = Config {
        repos: vec![PathBuf::from("from-config")],
        author: None,
    };
    let cli = vec![PathBuf::from("a"), PathBuf::from("b")];
    assert_eq!(effective_paths(cli.clone(), &cfg), cli);
}

#[test]
fn config_repos_used_when_no_cli_paths() {
    let cfg = Config {
        repos: vec![PathBuf::from("c1"), PathBuf::from("c2")],
        author: None,
    };
    assert_eq!(effective_paths(vec![], &cfg), cfg.repos);
}

#[test]
fn current_dir_used_when_no_cli_and_no_config() {
    assert_eq!(
        effective_paths(vec![], &Config::default()),
        vec![PathBuf::from(".")]
    );
}

#[test]
fn several_cli_paths_and_author_flag_forms_parse() {
    let a = parse_args(s(&["a", "b", "c", "--author", "x@y"])).unwrap();
    assert_eq!(a.paths.len(), 3);
    assert_eq!(a.author.as_deref(), Some("x@y"));
    let a = parse_args(s(&["--author=z@y", "a"])).unwrap();
    assert_eq!(a.author.as_deref(), Some("z@y"));
    assert_eq!(a.paths, vec![PathBuf::from("a")]);
    assert!(parse_args(s(&["--author"])).is_err());
    assert_eq!(parse_args(s(&[])).unwrap().author, None);
}

#[test]
fn config_file_supplies_repos_and_author() {
    let d = tempfile::tempdir().unwrap();
    let f = store::config_path(d.path());
    fs::write(
        &f,
        r#"{"repos":["C:\\Games\\One","D:\\Two"],"author":"me@x.y"}"#,
    )
    .unwrap();
    let (c, w) = repos::load_config(&f);
    assert!(w.is_none());
    assert_eq!(
        c.repos,
        vec![PathBuf::from(r"C:\Games\One"), PathBuf::from(r"D:\Two")]
    );
    assert_eq!(c.author.as_deref(), Some("me@x.y"));
}

// ---- author override ----

#[test]
fn only_the_given_author_feeds_in_every_repo() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let rs = repos::resolve(&[a.clone(), b.clone()]).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, "me@x.y", 0);
    commit(&a, "mine a");
    commit_as(&a, "other@x.y", "theirs a");
    commit_as(&b, "other@x.y", "theirs b");
    commit(&b, "mine b");
    assert_eq!(sync_all(&mut pet, &rs, "me@x.y", 1).fed, 2);

    // Override author: other's commits feed (a different pet, baselines kept).
    let mut pet2 = Pet::new(0);
    sync_all(&mut pet2, &rs, "other@x.y", 0);
    commit_as(&a, "other@x.y", "o1");
    commit_as(&b, "other@x.y", "o2");
    commit(&b, "mine again");
    assert_eq!(sync_all(&mut pet2, &rs, "other@x.y", 1).fed, 2);
}

// ---- parent-folder scan ----

#[test]
fn scan_depth_boundaries_and_skip_list() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("root");
    init(&p.join("d1"));
    init(&p.join("l1").join("d2"));
    init(&p.join("l1").join("l2").join("d3"));
    init(&p.join("l1").join("l2").join("l3").join("d4"));
    for skip in [
        "node_modules",
        "target",
        "venv",
        ".venv",
        "__pycache__",
        ".git2",
        ".cache",
    ] {
        init(&p.join(skip).join("inner"));
    }
    let r = repos::resolve(std::slice::from_ref(&p));
    assert_eq!(names(&r), ["d1", "d2", "d3"]);
    assert!(r.warnings.is_empty());
}

#[test]
fn scan_does_not_descend_into_repos_or_double_count_submodule_like_dirs() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("root");
    init(&p.join("outer"));
    init(&p.join("outer").join("vendor").join("inner"));
    let r = repos::resolve(std::slice::from_ref(&p));
    assert_eq!(names(&r), ["outer"]);
}

#[test]
fn non_repo_dirs_are_skipped_silently_in_a_scan() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("root");
    init(&p.join("r"));
    fs::create_dir_all(p.join("empty")).unwrap();
    fs::write(p.join("file.txt"), "x").unwrap();
    let r = repos::resolve(std::slice::from_ref(&p));
    assert_eq!(r.repos.len(), 1);
    assert!(r.warnings.is_empty());
}

#[test]
fn same_repo_via_two_paths_is_counted_once() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let r = repos::resolve(&[a.clone(), d.path().to_path_buf(), a.clone()]);
    assert_eq!(r.repos.len(), 1);
}

#[cfg(windows)]
#[test]
fn junctions_are_not_followed() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("root");
    let real = d.path().join("elsewhere").join("real");
    init(&real);
    fs::create_dir_all(&p).unwrap();
    init(&p.join("ok"));
    let out = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(p.join("link"))
        .arg(d.path().join("elsewhere"))
        .output()
        .unwrap();
    if !out.status.success() {
        return; // cannot create a junction here; nothing to assert
    }
    let r = repos::resolve(std::slice::from_ref(&p));
    assert_eq!(names(&r), ["ok"]);
}

// ---- per-repo baselines ----

#[test]
fn first_seen_repo_baselines_even_with_big_history() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    for i in 0..20 {
        commit(&a, &format!("h{i}"));
    }
    let rs = repos::resolve(std::slice::from_ref(&a)).repos;
    let mut pet = Pet::new(0);
    assert_eq!(sync_all(&mut pet, &rs, "me@x.y", 0).fed, 0);
    assert_eq!(pet.total_fed, 0);
    assert!(pet.repos.values().all(|t| t.initialized));
}

#[test]
fn rewritten_history_rebaselines_only_that_repo() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    commit(&a, "a1");
    commit(&b, "b1");
    let rs = repos::resolve(&[a.clone(), b.clone()]).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, "me@x.y", 0);
    commit(&a, "a2");
    commit(&b, "b2");
    // Simulate a's baseline vanishing (rewritten history + gc): garbage id.
    let key_a = Repo::open(&a).unwrap().key().to_string();
    pet.repos.get_mut(&key_a).unwrap().last_seen =
        Some("0123456789012345678901234567890123456789".into());
    commit(&a, "a-during-rewrite");
    let r = sync_all(&mut pet, &rs, "me@x.y", 1);
    assert!(r.errors.is_empty());
    assert_eq!(r.fed, 1, "b2 feeds; a is re-baselined and feeds nothing");
    commit(&a, "a-after");
    commit(&b, "b3");
    assert_eq!(sync_all(&mut pet, &rs, "me@x.y", 2).fed, 2);
}

// ---- shared pet across selections ----

#[test]
fn different_selections_across_runs_keep_one_pet_and_one_file() {
    let d = tempfile::tempdir().unwrap();
    let data = d.path().join("data");
    let (a, b, c) = (d.path().join("a"), d.path().join("b"), d.path().join("c"));
    init(&a);
    init(&b);
    init(&c);
    let save = store::shared_path(&data);
    let sel: [Vec<PathBuf>; 3] = [
        vec![a.clone()],
        vec![b.clone(), c.clone()],
        vec![a.clone(), c.clone()],
    ];
    let mut total = 0;
    for (i, paths) in sel.iter().enumerate() {
        let rs = repos::resolve(paths).repos;
        let mut pet = store::load(&save, i as i64);
        assert_eq!(pet.total_fed, total);
        sync_all(&mut pet, &rs, "me@x.y", i as i64);
        for p in paths {
            commit(p, "x");
            total += 1;
        }
        assert_eq!(
            sync_all(&mut pet, &rs, "me@x.y", i as i64 + 1).fed,
            paths.len()
        );
        store::save(&save, &pet).unwrap();
    }
    let files: Vec<_> = fs::read_dir(&data).unwrap().flatten().collect();
    assert_eq!(files.len(), 1, "exactly one save file");
    assert_eq!(store::load(&save, 99).total_fed as usize, total as usize);
}

// ---- failing / missing repos ----

#[test]
fn missing_and_non_repo_paths_warn_and_others_still_feed() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let plain = d.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let r = repos::resolve(&[d.path().join("gone"), plain, a.clone()]);
    assert_eq!(r.warnings.len(), 2);
    assert_eq!(names(&r), ["a"]);
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &r.repos, "me@x.y", 0);
    commit(&a, "x");
    assert_eq!(sync_all(&mut pet, &r.repos, "me@x.y", 1).fed, 1);
}

#[test]
fn no_resolvable_repo_gives_empty_result() {
    let d = tempfile::tempdir().unwrap();
    let r = repos::resolve(&[d.path().join("nope"), d.path().join("nope2")]);
    assert!(r.repos.is_empty());
    assert_eq!(r.warnings.len(), 2);
}

#[test]
#[ignore = "BUG: temporarily unreadable repo loses its baseline (last_seen=None) and refeeds history on return"]
fn temporarily_unavailable_repo_keeps_baseline() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    let rs = repos::resolve(&[a.clone(), b.clone()]).repos;
    let mut pet = Pet::new(0);
    sync_all(&mut pet, &rs, "me@x.y", 0);
    let hidden = d.path().join("b-hidden");
    fs::rename(&b, &hidden).unwrap();
    commit(&a, "a1");
    let r = sync_all(&mut pet, &rs, "me@x.y", 1);
    assert_eq!(r.fed, 1);
    fs::rename(&hidden, &b).unwrap();
    commit(&b, "b1");
    assert_eq!(sync_all(&mut pet, &rs, "me@x.y", 2).fed, 1);
}

// ---- legacy migration ----

fn legacy_body(head: &str) -> String {
    format!(
        r#"{{"hunger":40.0,"health":90.0,"total_fed":7,"last_update":10,
"initialized":true,"last_seen":"{head}","fed_ids":["{head}"],
"recent":[{{"id":"{head}","summary":"base","time":1}}]}}"#
    )
}

#[test]
fn legacy_save_with_commits_made_while_closed_feeds_only_those() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let repo = Repo::open(&a).unwrap();
    let data = d.path().join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(
        store::save_path(&data, Path::new(repo.key())),
        legacy_body(&repo.head().unwrap()),
    )
    .unwrap();
    commit(&a, "while closed 1");
    commit(&a, "while closed 2");
    let mut pet = store::migrate_legacy(&data, &[repo.key().to_string()], 11).unwrap();
    assert_eq!(pet.total_fed, 7, "not reset");
    assert_eq!(sync_all(&mut pet, &[repo], "me@x.y", 12).fed, 2);
    assert_eq!(pet.total_fed, 9);
}

#[test]
fn legacy_pet_kept_when_its_repo_not_selected_and_new_repo_only_baselines() {
    let d = tempfile::tempdir().unwrap();
    let (a, b) = (d.path().join("a"), d.path().join("b"));
    init(&a);
    init(&b);
    commit(&b, "old b history");
    let ra = Repo::open(&a).unwrap();
    let rb = Repo::open(&b).unwrap();
    let data = d.path().join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(
        store::save_path(&data, Path::new(ra.key())),
        legacy_body(&ra.head().unwrap()),
    )
    .unwrap();
    let mut pet = store::migrate_legacy(&data, &[rb.key().to_string()], 11).unwrap();
    assert_eq!(pet.total_fed, 7);
    assert_eq!(sync_all(&mut pet, &[rb], "me@x.y", 12).fed, 0);
}

#[test]
fn shared_save_present_blocks_migration_and_legacy_file_is_left_alone() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let repo = Repo::open(&a).unwrap();
    let data = d.path().join("data");
    fs::create_dir_all(&data).unwrap();
    let legacy = store::save_path(&data, Path::new(repo.key()));
    fs::write(&legacy, legacy_body(&repo.head().unwrap())).unwrap();
    fs::write(store::shared_path(&data), "{}").unwrap();
    assert!(store::migrate_legacy(&data, &[repo.key().to_string()], 1).is_none());
    assert!(legacy.exists());
}

#[test]
fn corrupt_legacy_saves_are_skipped_not_fatal() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a");
    init(&a);
    let repo = Repo::open(&a).unwrap();
    let data = d.path().join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(store::save_path(&data, Path::new(repo.key())), "not json").unwrap();
    assert!(store::migrate_legacy(&data, &[repo.key().to_string()], 1).is_none());
}

// ---- repos.json parsing ----

#[test]
fn repos_json_with_bom_loads() {
    let d = tempfile::tempdir().unwrap();
    let f = store::config_path(d.path());
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(br#"{"repos":["C:\\A"],"author":"me@x.y"}"#);
    fs::write(&f, bytes).unwrap();
    let (c, w) = repos::load_config(&f);
    assert!(w.is_none(), "{w:?}");
    assert_eq!(c.repos, vec![PathBuf::from(r"C:\A")]);
    assert_eq!(c.author.as_deref(), Some("me@x.y"));
}

#[test]
fn doubled_backslashes_decode_to_single() {
    let c = repos::parse_config(r#"{"repos":["C:\\Users\\me\\Tech & Coding"]}"#).unwrap();
    assert_eq!(c.repos, vec![PathBuf::from(r"C:\Users\me\Tech & Coding")]);
}

#[test]
fn single_backslash_paths_are_reported_not_silently_accepted() {
    // Invalid JSON escape (\U); spec requires a clear error rather than a panic.
    let r = repos::parse_config(r#"{"repos":["C:\Users\me"]}"#);
    assert!(r.is_err());
    assert!(r.unwrap_err().contains("repos.json"));
    // A single backslash before a valid escape letter is JSON-valid but still parses.
    assert!(repos::parse_config(r#"{"repos":["C:\\x"]}"#).is_ok());
}

#[test]
fn unparseable_repos_json_gives_warning_via_load_config() {
    let d = tempfile::tempdir().unwrap();
    let f = store::config_path(d.path());
    fs::write(&f, r#"{"repos": ["C:\Users\me"]"#).unwrap();
    let (c, w) = repos::load_config(&f);
    assert_eq!(c, Config::default());
    assert!(w.unwrap().contains("repos.json"));
    // Empty file is also unparseable.
    fs::write(&f, "").unwrap();
    assert!(repos::load_config(&f).1.is_some());
}

#[test]
fn partial_and_unknown_config_keys_are_tolerated() {
    assert_eq!(
        repos::parse_config(r#"{"author":"a"}"#)
            .unwrap()
            .repos
            .len(),
        0
    );
    assert_eq!(
        repos::parse_config(r#"{"repos":[],"extra":1}"#).unwrap(),
        Config::default()
    );
}

// ---- binary: fatal error, exit code 1 ----
// Paths are given on the command line, so run() reads (never writes) the
// config and fails before touching any save file.

#[test]
fn binary_exits_1_with_message_when_no_repo_resolves() {
    let d = tempfile::tempdir().unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_gitagochi"))
        .arg(d.path().join("does-not-exist"))
        .arg(d.path().join("also-missing"))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.starts_with("error:"), "{err}");
    assert!(err.contains("does-not-exist"), "{err}");
}

#[test]
fn binary_exits_1_for_plain_dir_with_no_repos() {
    let d = tempfile::tempdir().unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_gitagochi"))
        .arg(d.path())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&o.stderr).is_empty());
}
