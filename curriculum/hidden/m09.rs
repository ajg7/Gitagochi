use tracker::{Commit, Fields, longer, longest_summary, split_fields};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

fn within(outer: &str, inner: &str) -> bool {
    let (o, i) = (outer.as_ptr() as usize, inner.as_ptr() as usize);
    o <= i && i + inner.len() <= o + outer.len()
}

fn c(summary: &str) -> Commit {
    Commit {
        hash: "h".to_string(),
        author: "a".to_string(),
        time: 0,
        summary: summary.to_string(),
    }
}

#[test]
fn m09_split_fields_borrows() {
    let line = String::from("a1|ann@x.io|5|fix: a | b");
    let Some(f) = split_fields(&line) else {
        return check(false, "split_fields should accept a line with four fields");
    };
    let f: Fields<'_> = f;
    check(f.hash == "a1" && f.author == "ann@x.io" && f.time == "5", "split_fields should split hash, author and time");
    check(f.summary == "fix: a | b", "the summary should be everything after the third |");
    check(within(&line, f.author) && within(&line, f.summary), "the fields should be slices of the line, not copies");
    check(split_fields("a1|ann").is_none(), "a line with missing fields should give None");
}

#[test]
fn m09_longest_summary() {
    let v = vec![c("fix"), c("feat: login"), c("docs: readme"), c("docs: readmf")];
    let got = longest_summary(&v);
    check(got == Some("docs: readme"), "longest_summary should return the longest summary, the first on a tie");
    check(got.is_some_and(|s| within(&v[2].summary, s)), "longest_summary should borrow from the commits");
}

#[test]
fn m09_longest_summary_empty_none() {
    check(longest_summary(&[]).is_none(), "no commits should give None");
}

#[test]
fn m09_longer_annotated() {
    let a = String::from("short");
    let b = String::from("longer one");
    check(longer(&a, &b) == "longer one", "longer should return the longer string");
    check(longer("abc", "xyz") == "abc", "on a tie longer should return the first");
    check(within(&b, longer(&a, &b)), "longer should return one of its arguments, not a copy");
}
