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
}
