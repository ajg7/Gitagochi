//! Statistics over commits.

use crate::parse::{Commit, field};
use std::collections::BTreeMap;

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
        assert_eq!(
            count_lines(
                "a

b
"
            ),
            2
        );
    }

    #[test]
    fn streaks() {
        assert_eq!(longest_streak(&[1, 2, 2, 3, 7]), 3);
        assert_eq!(day_of(86_400), 1);
    }
}
