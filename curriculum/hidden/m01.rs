use tracker::count_lines;

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

#[test]
fn m01_counts_non_empty_lines() {
    let log = "a1|ann@x.io|100|first\n\nb2|bob@x.io|200|second\n   \nc3|ann@x.io|300|third";
    check(
        count_lines(log) == 3,
        "count_lines should count only the non-blank lines of the log",
    );
}

#[test]
fn m01_empty_log_is_zero() {
    check(count_lines("") == 0, "count_lines of an empty log should be 0");
}
