//! Gitagochi: a terminal pet that feeds on git commits.

pub mod git;
pub mod pet;
pub mod repos;
pub mod store;
pub mod ui;

use git::{GitError, Repo};
use pet::Pet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Current unix time in seconds (0 if the clock is before the epoch).
pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Apply decay, then feed the pet any new commits by `author` from one repo.
/// A repo's first sync only records a baseline (existing history does not
/// feed the pet). Returns the number of commits fed.
pub fn sync(pet: &mut Pet, repo: &Repo, author: &str, now: i64) -> Result<usize, GitError> {
    pet.tick(now);
    sync_repo(pet, repo, author)
}

/// Feed from one repo using its own tracking entry; no time decay.
pub fn sync_repo(pet: &mut Pet, repo: &Repo, author: &str) -> Result<usize, GitError> {
    let track = pet.repos.entry(repo.key().to_string()).or_default();
    if !track.initialized {
        track.last_seen = repo.head();
        track.initialized = true;
        return Ok(0);
    }
    let poll = repo.commits_since(track.last_seen.as_deref(), author)?;
    let fresh: Vec<_> = poll
        .new_commits
        .into_iter()
        .filter(|c| !track.has_fed(&c.id))
        .map(|mut c| {
            c.repo = repo.name().to_string();
            c
        })
        .collect();
    track.last_seen = poll.head;
    track.record(fresh.iter().rev().map(|c| c.id.clone()));
    pet.feed(&fresh);
    Ok(fresh.len())
}

/// Result of syncing several repos.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub fed: usize,
    pub errors: Vec<String>,
}

/// Apply decay once, then sync every repo; a failing repo does not stop others.
pub fn sync_all(pet: &mut Pet, repos: &[Repo], author: &str, now: i64) -> SyncReport {
    pet.tick(now);
    let mut report = SyncReport::default();
    for r in repos {
        match sync_repo(pet, r, author) {
            Ok(n) => report.fed += n,
            Err(e) => report.errors.push(format!("{}: {e}", r.name())),
        }
    }
    report
}

/// Command-line options.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub paths: Vec<PathBuf>,
    pub author: Option<String>,
    pub help: bool,
}

pub const USAGE: &str = "Usage: gitagochi [PATH ...] [--author EMAIL]

Runs a terminal pet that feeds on git commits. Each PATH is a git repo or a
parent folder holding repos (scanned 3 levels deep). With no PATH, repos come
from repos.json in the data dir; with neither, the current directory is used.
Commits by --author (default: config author, then git config user.email) feed the pet.
Keys: q quit, r refresh.";

pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Args, String> {
    let mut out = Args::default();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => out.help = true,
            "--author" => {
                out.author = Some(it.next().ok_or("--author needs a value")?);
            }
            s if s.starts_with("--author=") => {
                out.author = Some(s["--author=".len()..].to_string());
            }
            s if s.starts_with('-') => return Err(format!("unknown option: {s}")),
            _ => out.paths.push(PathBuf::from(a)),
        }
    }
    Ok(out)
}

/// Paths to watch: command line, else config, else the current directory.
pub fn effective_paths(cli: Vec<PathBuf>, config: &repos::Config) -> Vec<PathBuf> {
    if !cli.is_empty() {
        cli
    } else if !config.repos.is_empty() {
        config.repos.clone()
    } else {
        vec![PathBuf::from(".")]
    }
}

/// Entry point used by `main`. Errors are user-facing messages.
pub fn run(args: Args) -> Result<(), String> {
    let data_dir = store::default_data_dir();
    let (config, config_warning) = repos::load_config(&store::config_path(&data_dir));
    let author_arg = args.author.or_else(|| config.author.clone());
    let paths = effective_paths(args.paths, &config);
    let resolved = repos::resolve(&paths);
    if resolved.repos.is_empty() {
        let mut msgs = resolved.warnings;
        msgs.extend(config_warning);
        if msgs.is_empty() {
            msgs.push("no git repositories found".into());
        }
        return Err(msgs.join(
            "
",
        ));
    }
    let author = author_arg
        .or_else(|| resolved.repos.iter().find_map(Repo::default_author))
        .ok_or_else(|| GitError::NoAuthor.to_string())?;
    let save_file = store::shared_path(&data_dir);
    let now = now_unix();
    let keys: Vec<String> = resolved.repos.iter().map(|r| r.key().to_string()).collect();
    let pet = match store::migrate_legacy(&data_dir, &keys, now) {
        Some(p) => p,
        None => store::load(&save_file, now),
    };
    let session = ui::Session {
        paths,
        author,
        save_file,
        initial_warning: config_warning,
    };
    ui::run(session, resolved, pet).map_err(|e| format!("terminal error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_args() {
        let a = parse_args(s(&["foo", "--author", "a@b"])).unwrap();
        assert_eq!(a.paths, vec![PathBuf::from("foo")]);
        assert_eq!(a.author.as_deref(), Some("a@b"));
        assert!(parse_args(s(&["--author"])).is_err());
        assert!(parse_args(s(&["--nope"])).is_err());
        assert_eq!(parse_args(s(&["a", "b"])).unwrap().paths.len(), 2);
        assert_eq!(
            effective_paths(vec![], &repos::Config::default()),
            vec![PathBuf::from(".")]
        );
        assert!(parse_args(s(&["-h"])).unwrap().help);
    }
}
