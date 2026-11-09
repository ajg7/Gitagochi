//! tracker: reads a log of commits and reports stats about them.

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
