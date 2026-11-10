use tracker::{authors, field, normalize};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

#[test]
fn m03_field_returns_slice() {
    let line = String::from("a1|ann@x.io|100|fix the bug");
    check(field(&line, 0) == "a1", "field 0 should be the hash");
    check(field(&line, 1) == "ann@x.io", "field 1 should be the author");
    check(field(&line, 3) == "fix the bug", "field 3 should be the summary");
    let f = field(&line, 1);
    let inside = line.as_ptr() as usize <= f.as_ptr() as usize
        && f.as_ptr() as usize <= line.as_ptr() as usize + line.len();
    check(inside, "field should return a slice borrowed from the line, not a copy");
}

#[test]
fn m03_field_missing_is_empty() {
    check(field("a1|ann", 3).is_empty(), "a missing field should be \"\"");
    check(field("", 0).is_empty(), "field 0 of an empty line should be \"\"");
}

#[test]
fn m03_authors_unique_in_order() {
    let log = "a1|bob@x.io|1|x\nb2|ann@x.io|2|y\n\nc3|bob@x.io|3|z\nd4|cy@x.io|4|w\n";
    let got = authors(log);
    let want = vec!["bob@x.io".to_string(), "ann@x.io".to_string(), "cy@x.io".to_string()];
    check(got == want, "authors should list each author once, in first-seen order");
    check(authors("").is_empty(), "an empty log should have no authors");
}

#[test]
fn m03_normalize_lowercases_in_place() {
    let mut names = vec!["Ann@X.io".to_string(), "bob@x.io".to_string()];
    normalize(&mut names);
    check(names[0] == "ann@x.io", "normalize should lowercase each name");
    check(names[1] == "bob@x.io", "normalize should keep lowercase names unchanged");
    check(names.len() == 2, "normalize should not add or remove names");
}
