//! tracker: reads a log of commits and reports stats about them.

/// Number of non-blank lines in `log`.
pub fn count_lines(log: &str) -> usize {
    log.lines().filter(|l| !l.trim().is_empty()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_lines() {
        assert_eq!(count_lines("a\n\nb\n"), 2);
    }
}
