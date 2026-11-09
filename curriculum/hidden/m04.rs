use tracker::{Commit, Kind, parse_commit};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

fn commit(summary: &str) -> Commit {
    Commit {
        hash: "a1".to_string(),
        author: "ann@x.io".to_string(),
        time: 5,
        summary: summary.to_string(),
    }
}

#[test]
fn m04_parse_commit_fields() {
    let got = parse_commit("a1|ann@x.io|5|fix: a | b");
    check(got == Some(commit("fix: a | b")), "parse_commit should fill every field; the summary keeps any later |");
    let c = commit("x");
    check(c.clone() == c, "Commit should derive Clone and PartialEq");
}

#[test]
fn m04_parse_commit_rejects_short_line() {
    check(parse_commit("a1|ann@x.io|5").is_none(), "a line without a summary field should give None");
    check(parse_commit("").is_none(), "an empty line should give None");
    check(parse_commit("a1|ann@x.io|soon|x").is_none(), "a time that is not an integer should give None");
}

#[test]
fn m04_kind_from_prefix() {
    check(commit("feat: login").kind() == Kind::Feature, "a summary starting with feat should be Kind::Feature");
    check(commit("fix(ui): crash").kind() == Kind::Fix, "a summary starting with fix should be Kind::Fix");
    check(commit("docs: readme").kind() == Kind::Docs, "a summary starting with docs should be Kind::Docs");
    check(commit("refactor").kind() == Kind::Other, "any other summary should be Kind::Other");
}
