//! Repo selection: config file parsing and resolving paths to repos.

use crate::git::Repo;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// How many directory levels below a parent folder are scanned for repos.
pub const MAX_SCAN_DEPTH: usize = 3;

const SKIP_DIRS: [&str; 6] = [
    "node_modules",
    "target",
    "venv",
    ".venv",
    "__pycache__",
    ".git",
];

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub repos: Vec<PathBuf>,
    pub author: Option<String>,
}

pub fn parse_config(text: &str) -> Result<Config, String> {
    serde_json::from_str(text).map_err(|e| format!("invalid repos.json: {e}"))
}

/// Read `repos.json`. Missing file gives the default config; a bad file gives
/// the default config plus a warning.
pub fn load_config(path: &Path) -> (Config, Option<String>) {
    match fs::read_to_string(path) {
        Ok(t) => match parse_config(&t) {
            Ok(c) => (c, None),
            Err(e) => (Config::default(), Some(e)),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => (Config::default(), None),
        Err(e) => (
            Config::default(),
            Some(format!("cannot read repos.json: {e}")),
        ),
    }
}

pub struct Resolved {
    pub repos: Vec<Repo>,
    pub warnings: Vec<String>,
}

fn is_repo_dir(p: &Path) -> bool {
    p.join(".git").exists()
}

fn is_link(md: &fs::Metadata) -> bool {
    if md.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const REPARSE_POINT: u32 = 0x400;
        if md.file_attributes() & REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

fn skipped(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIRS.contains(&name)
}

fn scan(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut subs: Vec<PathBuf> = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if skipped(&name) {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        if !md.is_dir() || is_link(&md) {
            continue;
        }
        subs.push(e.path());
    }
    subs.sort();
    for p in subs {
        if is_repo_dir(&p) {
            out.push(p);
        } else if depth < MAX_SCAN_DEPTH {
            scan(&p, depth + 1, out);
        }
    }
}

/// Resolve each path to repos: a repo itself, a parent folder holding repos
/// (scanned `MAX_SCAN_DEPTH` levels deep, not descending into repos), or a
/// folder inside a repo. Problems become warnings; duplicates are dropped.
pub fn resolve(paths: &[PathBuf]) -> Resolved {
    let mut repos: Vec<Repo> = Vec::new();
    let mut warnings = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut add = |r: Repo, repos: &mut Vec<Repo>| {
        if seen.insert(r.key().to_lowercase()) {
            repos.push(r);
        }
    };
    for p in paths {
        if !p.is_dir() {
            warnings.push(format!("skipped {}: no such directory", p.display()));
            continue;
        }
        if is_repo_dir(p) {
            add(Repo::from_root(p.clone()), &mut repos);
            continue;
        }
        let mut found = Vec::new();
        scan(p, 1, &mut found);
        if found.is_empty() {
            match Repo::open(p) {
                Ok(r) => add(r, &mut repos),
                Err(_) => warnings.push(format!("skipped {}: not a git repo", p.display())),
            }
        } else {
            for f in found {
                add(Repo::from_root(f), &mut repos);
            }
        }
    }
    Resolved { repos, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(p: &Path) {
        fs::create_dir_all(p.join(".git")).unwrap();
    }

    fn names(r: &Resolved) -> Vec<String> {
        let mut v: Vec<String> = r.repos.iter().map(|r| r.name().to_string()).collect();
        v.sort();
        v
    }

    #[test]
    fn parses_config() {
        let c = parse_config(r#"{"repos":["C:\\A & B\\x"],"author":"a@b"}"#).unwrap();
        assert_eq!(c.repos, vec![PathBuf::from(r"C:\A & B\x")]);
        assert_eq!(c.author.as_deref(), Some("a@b"));
        assert_eq!(parse_config("{}").unwrap(), Config::default());
        assert!(parse_config("[1").is_err());
    }

    #[test]
    fn bad_config_file_warns() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("repos.json");
        assert_eq!(load_config(&f), (Config::default(), None));
        fs::write(&f, "nope").unwrap();
        assert!(load_config(&f).1.is_some());
    }

    #[test]
    fn explicit_repo_and_missing_path() {
        let d = tempfile::tempdir().unwrap();
        mk(&d.path().join("a"));
        let r = resolve(&[d.path().join("a"), d.path().join("gone")]);
        assert_eq!(names(&r), ["a"]);
        assert_eq!(r.warnings.len(), 1);
    }

    #[test]
    fn scans_three_levels_and_skips_junk() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("Tech & Coding");
        mk(&p.join("one"));
        mk(&p.join("g").join("two"));
        mk(&p.join("g").join("h").join("three"));
        mk(&p.join("g").join("h").join("i").join("four"));
        mk(&p.join("one").join("nested"));
        mk(&p.join("node_modules").join("dep"));
        mk(&p.join("target").join("dep"));
        mk(&p.join(".hidden").join("dep"));
        fs::create_dir_all(p.join("plain").join("dir")).unwrap();
        let r = resolve(std::slice::from_ref(&p));
        assert_eq!(names(&r), ["one", "three", "two"]);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn empty_parent_warns() {
        let d = tempfile::tempdir().unwrap();
        let r = resolve(&[d.path().to_path_buf()]);
        assert!(r.repos.is_empty());
        assert_eq!(r.warnings.len(), 1);
    }

    #[test]
    fn duplicates_dropped() {
        let d = tempfile::tempdir().unwrap();
        mk(&d.path().join("a"));
        let r = resolve(&[d.path().join("a"), d.path().to_path_buf()]);
        assert_eq!(r.repos.len(), 1);
    }
}
