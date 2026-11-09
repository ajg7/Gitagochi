use tracker::{ParseError, parse_line, parse_log};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

#[test]
fn m05_parse_log_ok() {
    let log = "a1|ann@x.io|5|feat: one\n\nb2|bob@x.io|9|fix: two\n";
    match parse_log(log) {
        Ok(v) => {
            check(v.len() == 2, "parse_log should return one Commit per non-blank line");
            check(v[0].hash == "a1" && v[1].author == "bob@x.io", "parse_log should keep the log order");
        }
        Err(_) => check(false, "parse_log should succeed on a valid log"),
    }
    check(parse_log("") == Ok(vec![]), "an empty log should give Ok with no commits");
    check(parse_line("a1|ann@x.io|5|x", 1).is_ok(), "parse_line should accept a valid line");
}

#[test]
fn m05_reports_missing_field_line() {
    let log = "a1|ann@x.io|5|ok\n\nb2|bob@x.io\n";
    check(parse_log(log) == Err(ParseError::MissingField(3)), "a short line 3 should give Err(MissingField(3)), counting the blank line");
    check(parse_line("a1", 7) == Err(ParseError::MissingField(7)), "parse_line should report the line number it was given");
}

#[test]
fn m05_reports_bad_time_line() {
    let log = "a1|ann@x.io|5|ok\nb2|bob@x.io|later|x\nc3|cy\n";
    check(parse_log(log) == Err(ParseError::BadTime(2)), "parse_log should return the first error: BadTime(2)");
    check(parse_line("a1|ann@x.io|-|x", 4) == Err(ParseError::BadTime(4)), "a time that is not an integer should give BadTime");
}
