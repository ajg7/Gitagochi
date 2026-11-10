use tracker::pet::{Mood, Pet};
use tracker::{Commit, ParseError};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

fn describe(e: &dyn std::error::Error) -> String {
    e.to_string()
}

#[test]
fn m10_pet_default_and_feed_saturates() {
    let mut p = Pet::default();
    check(p.hunger() == 50, "a default Pet should have hunger 50");
    p.feed(2);
    check(p.hunger() == 30, "feeding 2 commits should lower hunger by 20");
    p.feed(usize::MAX);
    check(p.hunger() == 0, "feeding a huge number should stop at 0 without panicking");
    p.starve(3);
    check(p.hunger() == 30, "starving 3 hours should raise hunger by 30");
    p.starve(u32::MAX);
    check(p.hunger() == 100, "starving a huge number of hours should stop at 100 without panicking");
}

#[test]
fn m10_mood_display() {
    let mut p = Pet::default();
    check(p.mood() == Mood::Sleeping, "hunger 50 should be Mood::Sleeping");
    p.feed(3);
    check(p.mood() == Mood::Happy, "hunger 20 should be Mood::Happy");
    p.starve(6);
    check(p.mood() == Mood::Hungry, "hunger 80 should be Mood::Hungry");
    check(Mood::Happy.to_string() == "happy", "Mood::Happy should display as \"happy\"");
    check(format!("{}", Mood::Hungry) == "hungry", "Mood::Hungry should display as \"hungry\"");
    check(Mood::Sleeping.to_string() == "sleeping", "Mood::Sleeping should display as \"sleeping\"");
}

#[test]
fn m10_commit_from_str() {
    let want = Commit {
        hash: "a1".to_string(),
        author: "ann".to_string(),
        time: 5,
        summary: "x".to_string(),
    };
    check("a1|ann|5|x".parse::<Commit>() == Ok(want), "parse::<Commit>() should build the Commit");
    check("bad".parse::<Commit>() == Err(ParseError::MissingField(1)), "a bad line should give Err(MissingField(1))");
}

#[test]
fn m10_parse_error_is_std_error() {
    let text = describe(&ParseError::MissingField(3));
    check(text.contains("line 3"), "ParseError's Display should name the line, like \"line 3: missing field\"");
    let other = describe(&ParseError::BadTime(4));
    check(other.contains("line 4") && other != text, "each ParseError variant should have its own message");
}
