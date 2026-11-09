//! tracker: reads a log of commits and reports stats about them.

use std::collections::BTreeMap;
use std::fmt;

pub const SECONDS_PER_DAY: i64 = 86_400;

/// Number of non-blank lines in `log`.
pub fn count_lines(log: &str) -> usize {
    log.lines().filter(|l| !l.trim().is_empty()).count()
}

/// Whole days since the unix epoch for a timestamp in seconds.
pub fn day_of(ts: i64) -> i64 {
    ts / SECONDS_PER_DAY
}

/// Longest run of consecutive days in a sorted list that may repeat days.
pub fn longest_streak(days: &[i64]) -> u32 {
    if days.is_empty() {
        return 0;
    }
    let mut best = 1;
    let mut run = 1;
    for pair in days.windows(2) {
        if pair[1] == pair[0] + 1 {
            run += 1;
        } else if pair[1] != pair[0] {
            run = 1;
        }
        best = best.max(run);
    }
    best
}

/// The `i`-th `|`-separated field of `line`, or "" if missing.
pub fn field(line: &str, i: usize) -> &str {
    line.split('|').nth(i).unwrap_or("")
}

/// Distinct authors of the log, in first-seen order.
pub fn authors(log: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in log.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let a = field(line, 1);
        if !out.iter().any(|x| x == a) {
            out.push(a.to_string());
        }
    }
    out
}

/// Lowercase every name in place.
pub fn normalize(names: &mut [String]) {
    for name in names.iter_mut() {
        *name = name.to_lowercase();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub time: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Feature,
    Fix,
    Docs,
    Other,
}

impl Commit {
    /// The kind of change, from the summary's prefix.
    pub fn kind(&self) -> Kind {
        match self.summary.as_str() {
            s if s.starts_with("feat") => Kind::Feature,
            s if s.starts_with("fix") => Kind::Fix,
            s if s.starts_with("docs") => Kind::Docs,
            _ => Kind::Other,
        }
    }
}

/// Parse `hash|author|time|summary`; None if a field is missing or bad.
pub fn parse_commit(line: &str) -> Option<Commit> {
    let mut parts = line.splitn(4, '|');
    let (Some(hash), Some(author), Some(time), Some(summary)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    Some(Commit {
        hash: hash.to_string(),
        author: author.to_string(),
        time: time.parse().ok()?,
        summary: summary.to_string(),
    })
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingField(usize),
    BadTime(usize),
}

/// Parse line number `n` (1-based) of a log.
pub fn parse_line(line: &str, n: usize) -> Result<Commit, ParseError> {
    let mut parts = line.splitn(4, '|');
    let (Some(hash), Some(author), Some(time), Some(summary)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::MissingField(n));
    };
    let time = time.parse::<i64>().map_err(|_| ParseError::BadTime(n))?;
    Ok(Commit {
        hash: hash.to_string(),
        author: author.to_string(),
        time,
        summary: summary.to_string(),
    })
}

/// Parse every non-blank line, stopping at the first error.
pub fn parse_log(log: &str) -> Result<Vec<Commit>, ParseError> {
    let mut out = Vec::new();
    for (i, line) in log.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(parse_line(line, i + 1)?);
    }
    Ok(out)
}

/// How many commits each author made.
pub fn commits_per_author(commits: &[Commit]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for c in commits {
        *counts.entry(c.author.clone()).or_insert(0) += 1;
    }
    counts
}

/// The `n` most active authors, most commits first, ties by name.
pub fn top_authors(commits: &[Commit], n: usize) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> = commits_per_author(commits).into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}

/// Longest run of consecutive days with a commit, in any input order.
pub fn streak(commits: &[Commit]) -> u32 {
    let mut days: Vec<i64> = commits.iter().map(|c| day_of(c.time)).collect();
    days.sort();
    longest_streak(&days)
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}: {}", self.hash, self.author, self.summary)
    }
}

/// A named statistic over a list of commits.
pub trait Stat {
    fn name(&self) -> String;
    fn value(&self, commits: &[Commit]) -> String;
}

pub struct TotalCommits;

impl Stat for TotalCommits {
    fn name(&self) -> String {
        "total commits".to_string()
    }

    fn value(&self, commits: &[Commit]) -> String {
        commits.len().to_string()
    }
}

pub struct LongestStreak;

impl Stat for LongestStreak {
    fn name(&self) -> String {
        "longest streak".to_string()
    }

    fn value(&self, commits: &[Commit]) -> String {
        streak(commits).to_string()
    }
}

/// One "name: value" line per stat.
pub fn report(stats: &[&dyn Stat], commits: &[Commit]) -> Vec<String> {
    stats
        .iter()
        .map(|s| format!("{}: {}", s.name(), s.value(commits)))
        .collect()
}

/// The most frequent value; ties go to the smallest.
pub fn most_common<T: Ord + Clone>(items: &[T]) -> Option<T> {
    let mut counts: BTreeMap<&T, usize> = BTreeMap::new();
    for x in items {
        *counts.entry(x).or_insert(0) += 1;
    }
    let mut best: Option<(&T, usize)> = None;
    for (x, n) in counts {
        if best.is_none_or(|(_, b)| n > b) {
            best = Some((x, n));
        }
    }
    best.map(|(x, _)| x.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_lines() {
        assert_eq!(count_lines("a\n\nb\n"), 2);
    }

    #[test]
    fn streaks() {
        assert_eq!(longest_streak(&[1, 2, 2, 3, 7]), 3);
        assert_eq!(day_of(86_400), 1);
    }

    #[test]
    fn fields_and_authors() {
        assert_eq!(field("a|b", 1), "b");
        assert_eq!(authors("1|x|0|s\n2|x|0|t\n"), vec!["x".to_string()]);
    }
}
