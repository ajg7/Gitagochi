//! Git commit ingestion by shelling out to the `git` binary.

use crate::pet::FedCommit;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Upper bound on commits read in one poll, to bound work on huge histories.
const MAX_NEW_COMMITS: usize = 1000;
/// Commit summaries are truncated to this many characters on ingest.
const MAX_SUMMARY_CHARS: usize = 200;

#[derive(Debug)]
pub enum GitError {
    GitNotFound,
    NotARepo(PathBuf),
    NoAuthor,
    Command(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::GitNotFound => write!(f, "could not run `git`; is it installed and on PATH?"),
            GitError::NotARepo(p) => write!(f, "{} is not inside a git repository", p.display()),
            GitError::NoAuthor => write!(
                f,
                "no author configured; set `git config user.email` or pass --author <email>"
            ),
            GitError::Command(s) => write!(f, "git failed: {s}"),
        }
    }
}

impl std::error::Error for GitError {}

/// Result of polling a repo.
#[derive(Debug, PartialEq, Eq)]
pub struct Poll {
    /// Current HEAD, `None` if the repo has no commits.
    pub head: Option<String>,
    /// New commits by the author since the baseline, newest first.
    pub new_commits: Vec<FedCommit>,
}

pub struct Repo {
    root: PathBuf,
    key: String,
    name: String,
}

fn run(dir: &Path, args: &[&str]) -> Result<std::process::Output, GitError> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|_| GitError::GitNotFound)
}

fn stdout(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let out = run(dir, args)?;
    if !out.status.success() {
        return Err(GitError::Command(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

impl Repo {
    /// Open the repo containing `path`; clear error if it is not a git repo.
    pub fn open(path: &Path) -> Result<Repo, GitError> {
        if !path.is_dir() {
            return Err(GitError::NotARepo(path.to_path_buf()));
        }
        let out = run(path, &["rev-parse", "--show-toplevel"])?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if err.is_empty() || err.contains("not a git repository") {
                return Err(GitError::NotARepo(path.to_path_buf()));
            }
            return Err(GitError::Command(err));
        }
        let top = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(Repo::from_root(PathBuf::from(top)))
    }

    /// Build a repo handle without running git (caller saw a `.git` entry).
    pub fn from_root(root: PathBuf) -> Repo {
        let key = root
            .canonicalize()
            .unwrap_or_else(|_| root.clone())
            .to_string_lossy()
            .into_owned();
        let name = root
            .file_name()
            .map(|n| strip_control(&n.to_string_lossy()))
            .unwrap_or_else(|| "repo".into());
        Repo { root, key, name }
    }

    /// Canonical path string identifying this repo in the save file.
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The configured `user.email`, if any.
    pub fn default_author(&self) -> Option<String> {
        let s = stdout(&self.root, &["config", "user.email"]).ok()?;
        let s = s.trim();
        (!s.is_empty()).then(|| s.to_string())
    }

    pub fn head(&self) -> Option<String> {
        let out = run(&self.root, &["rev-parse", "--verify", "-q", "HEAD"]).ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    }

    /// Commits by `author_email` reachable from HEAD but not from `since`.
    /// With `since == None`, all commits reachable from HEAD (bounded).
    /// If `since` is no longer valid (history rewritten), returns no commits.
    /// Other git failures are returned as errors.
    pub fn commits_since(&self, since: Option<&str>, author_email: &str) -> Result<Poll, GitError> {
        let head = self.head();
        if head.is_none() || head.as_deref() == since {
            return Ok(Poll {
                head,
                new_commits: Vec::new(),
            });
        }
        if let Some(s) = since {
            let check = format!("{s}^{{commit}}");
            let ok = run(&self.root, &["rev-parse", "--verify", "--quiet", &check])?;
            if !ok.status.success() {
                let err = String::from_utf8_lossy(&ok.stderr).trim().to_string();
                if !err.is_empty() {
                    return Err(GitError::Command(err));
                }
                return Ok(Poll {
                    head,
                    new_commits: Vec::new(),
                });
            }
        }
        let range = match since {
            Some(s) => format!("{s}..HEAD"),
            None => "HEAD".to_string(),
        };
        let author = format!("--author=<{author_email}>");
        let max = format!("--max-count={MAX_NEW_COMMITS}");
        let out = run(
            &self.root,
            &[
                "log",
                "--fixed-strings",
                "--regexp-ignore-case",
                &max,
                &author,
                "--format=%H%x1f%ct%x1f%s",
                &range,
                "--",
            ],
        )?;
        if !out.status.success() {
            return Err(GitError::Command(
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ));
        }
        Ok(Poll {
            head,
            new_commits: parse_log(&String::from_utf8_lossy(&out.stdout)),
        })
    }
}

fn strip_control(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '' {
            if it.peek() == Some(&'[') {
                it.next();
                for n in it.by_ref() {
                    if ('@'..='~').contains(&n) {
                        break;
                    }
                }
            }
        } else if !c.is_control() {
            out.push(c);
        }
    }
    out
}

fn parse_log(text: &str) -> Vec<FedCommit> {
    text.lines()
        .filter_map(|line| {
            let mut it = line.splitn(3, '\x1f');
            let id = it.next()?.trim();
            let time = it.next()?.trim().parse().ok()?;
            let summary: String = strip_control(it.next().unwrap_or(""))
                .chars()
                .take(MAX_SUMMARY_CHARS)
                .collect();
            (!id.is_empty()).then(|| FedCommit {
                id: id.to_string(),
                summary,
                time,
                repo: String::new(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_summaries() {
        let long = "é".repeat(500);
        let v = parse_log(&format!(
            "abc1{long}
"
        ));
        assert_eq!(v[0].summary.chars().count(), MAX_SUMMARY_CHARS);
    }

    #[test]
    fn strips_control_and_ansi() {
        let raw = [
            "abc",
            "\u{1f}",
            "1",
            "\u{1f}",
            "\u{1b}[31mred\u{1b}[0m",
            "\u{7} t",
            "\t",
            "x",
        ]
        .concat();
        let v = parse_log(&raw);
        assert_eq!(v[0].summary, "red tx");
    }

    #[test]
    fn parses_log_lines() {
        let v = parse_log("abc\x1f123\x1fhello world\nbad line\ndef\x1f5\x1f\n");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].id, "abc");
        assert_eq!(v[0].time, 123);
        assert_eq!(v[0].summary, "hello world");
        assert_eq!(v[1].summary, "");
    }
}
