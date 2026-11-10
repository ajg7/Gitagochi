//! tracker: reads a log of commits and reports stats about them.

pub mod parse;
pub mod stats;

pub use parse::{Commit, Kind, ParseError, field, parse_commit, parse_line, parse_log};
pub use stats::{
    LongestStreak, SECONDS_PER_DAY, Stat, TotalCommits, authors, commits_per_author, count_lines,
    day_of, longest_streak, most_common, normalize, report, streak, top_authors,
};
