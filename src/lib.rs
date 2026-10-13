//! Gitagochi: a terminal pet that feeds on git commits.

pub mod git;
pub mod pet;
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

/// Apply decay, then feed the pet any new commits by `author`.
/// The first sync on a repo only records a baseline (existing history does
/// not feed the pet). Returns the number of commits fed.
pub fn sync(pet: &mut Pet, repo: &Repo, author: &str, now: i64) -> Result<usize, GitError> {
    pet.tick(now);
    if !pet.initialized {
        pet.last_seen = repo.head();
        pet.initialized = true;
        return Ok(0);
    }
    let poll = repo.commits_since(pet.last_seen.as_deref(), author)?;
    let known: std::collections::HashSet<&str> = pet.fed_ids.iter().map(String::as_str).collect();
    let fresh: Vec<_> = poll
        .new_commits
        .into_iter()
        .filter(|c| !known.contains(c.id.as_str()))
        .collect();
    pet.feed(&fresh);
    pet.last_seen = poll.head;
    Ok(fresh.len())
}

/// Command-line options.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub path: Option<PathBuf>,
    pub author: Option<String>,
    pub help: bool,
}

pub const USAGE: &str = "Usage: gitagochi [PATH] [--author EMAIL]\n\n\
Runs a terminal pet that feeds on git commits in the repo at PATH (default: .).\n\
Commits by --author (default: git config user.email) feed the pet.\n\
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
            _ => {
                if out.path.is_some() {
                    return Err("only one PATH may be given".into());
                }
                out.path = Some(PathBuf::from(a));
            }
        }
    }
    Ok(out)
}

/// Entry point used by `main`. Errors are user-facing messages.
pub fn run(args: Args) -> Result<(), String> {
    let path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let repo = Repo::open(&path).map_err(|e| e.to_string())?;
    let author = args
        .author
        .or_else(|| repo.default_author())
        .ok_or_else(|| GitError::NoAuthor.to_string())?;
    let canonical = repo
        .root()
        .canonicalize()
        .unwrap_or_else(|_| repo.root().to_path_buf());
    let save_file = store::save_path(&store::default_data_dir(), &canonical);
    let pet = store::load(&save_file, now_unix());
    ui::run(&repo, &author, &save_file, pet).map_err(|e| format!("terminal error: {e}"))
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
        assert_eq!(a.path, Some(PathBuf::from("foo")));
        assert_eq!(a.author.as_deref(), Some("a@b"));
        assert!(parse_args(s(&["--author"])).is_err());
        assert!(parse_args(s(&["--nope"])).is_err());
        assert!(parse_args(s(&["a", "b"])).is_err());
        assert!(parse_args(s(&["-h"])).unwrap().help);
    }
}
