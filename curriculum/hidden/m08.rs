use tracker::parse::{Commit, Kind, ParseError, parse_commit, parse_line, parse_log};
use tracker::stats::{
    LongestStreak, Stat, TotalCommits, commits_per_author, count_lines, day_of, longest_streak,
    most_common, report, streak, top_authors,
};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

#[test]
fn m08_parse_module_path() {
    let c: Option<Commit> = parse_commit("a1|ann|5|feat: x");
    check(c.map(|c| c.kind()) == Some(Kind::Feature), "tracker::parse should hold Commit, Kind and parse_commit");
    check(parse_line("a1", 2) == Err(ParseError::MissingField(2)), "tracker::parse should hold parse_line and ParseError");
    check(parse_log("").is_ok(), "tracker::parse should hold parse_log");
    let old: Option<tracker::Commit> = tracker::parse_commit("a1|ann|5|x");
    check(old.is_some(), "the old path tracker::parse_commit should still work through pub use");
}

#[test]
fn m08_stats_module_path() {
    let v = parse_log("a1|ann|5|x\nb2|bob|86405|y\n").unwrap_or_default();
    check(count_lines("a\n") == 1 && day_of(86_400) == 1, "tracker::stats should hold count_lines and day_of");
    check(longest_streak(&[1, 2]) == 2 && streak(&v) == 2, "tracker::stats should hold longest_streak and streak");
    check(commits_per_author(&v).len() == 2 && top_authors(&v, 1).len() == 1, "tracker::stats should hold the author stats");
    check(most_common(&[1, 1]) == Some(1), "tracker::stats should hold most_common");
    check(TotalCommits.name() == "total commits" && LongestStreak.value(&v) == "2", "tracker::stats should hold the Stat types");
    check(report(&[&TotalCommits], &v) == vec!["total commits: 2".to_string()], "tracker::stats should hold report");
    check(tracker::top_authors(&v, 5).len() == 2, "the old path tracker::top_authors should still work through pub use");
}
