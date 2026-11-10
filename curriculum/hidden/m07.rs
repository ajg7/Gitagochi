use tracker::{Commit, LongestStreak, Stat, TotalCommits, most_common, report};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

fn c(author: &str, day: i64, summary: &str) -> Commit {
    Commit {
        hash: format!("h{day}"),
        author: author.to_string(),
        time: day * 86_400,
        summary: summary.to_string(),
    }
}

struct Authors;

impl Stat for Authors {
    fn name(&self) -> String {
        "authors".to_string()
    }
    fn value(&self, commits: &[Commit]) -> String {
        let mut v: Vec<&str> = commits.iter().map(|c| c.author.as_str()).collect();
        v.sort();
        v.dedup();
        v.len().to_string()
    }
}

#[test]
fn m07_display_commit() {
    let s = c("ann@x.io", 1, "fix crash").to_string();
    check(s == "h1 ann@x.io: fix crash", "Display for Commit should print \"hash author: summary\"");
}

#[test]
fn m07_stats_report() {
    let v = vec![c("a", 1, "x"), c("b", 2, "y"), c("a", 5, "z")];
    let lines = report(&[&TotalCommits, &LongestStreak, &Authors], &v);
    let want = vec!["total commits: 3".to_string(), "longest streak: 2".to_string(), "authors: 2".to_string()];
    check(lines == want, "report should give one \"name: value\" line per stat, in order, for any type implementing Stat");
    check(report(&[], &v).is_empty(), "no stats should give no lines");
}

#[test]
fn m07_most_common_generic() {
    check(most_common(&[3, 1, 3, 2]) == Some(3), "most_common should find the most frequent number");
    check(most_common(&["b", "a", "b", "a"]) == Some("a"), "on a tie, most_common should return the smallest value");
    check(most_common::<u8>(&[]).is_none(), "an empty slice should give None");
    let names = vec!["x".to_string(), "y".to_string(), "y".to_string()];
    check(most_common(&names) == Some("y".to_string()), "most_common should work for Strings too");
}
