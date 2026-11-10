use tracker::parse::parse_log;
use tracker::stats::{Stat, TotalCommits};

#[test]
fn total_of_parsed_log() {
    let commits = parse_log("a|x|1|s\nb|y|2|t\n").unwrap();
    assert_eq!(TotalCommits.value(&commits), "2");
}
