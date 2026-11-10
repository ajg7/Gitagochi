//! tracker: reads a log of commits and reports stats about them.

pub mod parse;
pub mod pet;
pub mod stats;

pub use parse::{
    Commit, Fields, Kind, ParseError, field, parse_commit, parse_line, parse_log, split_fields,
};
pub use stats::{
    LongestStreak, SECONDS_PER_DAY, Stat, TotalCommits, authors, commits_per_author, count_lines,
    day_of, longer, longest_streak, longest_summary, most_common, normalize, report, streak,
    top_authors,
};
