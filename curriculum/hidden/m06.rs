use tracker::{Commit, commits_per_author, streak, top_authors};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

fn c(author: &str, day: i64) -> Commit {
    Commit {
        hash: format!("{author}{day}"),
        author: author.to_string(),
        time: day * 86_400 + 3_600,
        summary: "x".to_string(),
    }
}

#[test]
fn m06_counts_per_author() {
    let v = vec![c("bob", 1), c("ann", 2), c("bob", 3)];
    let m = commits_per_author(&v);
    check(m.len() == 2, "commits_per_author should have one entry per author");
    check(m.get("bob") == Some(&2), "bob made 2 commits");
    check(m.get("ann") == Some(&1), "ann made 1 commit");
    check(commits_per_author(&[]).is_empty(), "no commits should give an empty map");
}

#[test]
fn m06_top_authors_sorted() {
    let v = vec![c("cy", 1), c("bob", 1), c("ann", 1), c("bob", 2), c("cy", 2), c("dee", 3), c("cy", 3)];
    let want = vec![("cy".to_string(), 3), ("bob".to_string(), 2), ("ann".to_string(), 1)];
    check(top_authors(&v, 3) == want, "top_authors should sort by count descending, then name ascending, and keep n");
    check(top_authors(&v, 10).len() == 4, "asking for more authors than exist should return them all");
    check(top_authors(&v, 0).is_empty(), "n = 0 should return no authors");
}

#[test]
fn m06_streak_from_commits() {
    let v = vec![c("a", 12), c("a", 10), c("b", 11), c("a", 11), c("a", 20)];
    check(streak(&v) == 3, "unsorted commits on days 10, 11, 11, 12 should give a streak of 3");
    check(streak(&[]) == 0, "no commits should give a streak of 0");
}
