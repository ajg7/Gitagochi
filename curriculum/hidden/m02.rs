use tracker::{day_of, longest_streak};

fn check(ok: bool, msg: &str) {
    if !ok {
        panic!("{msg}");
    }
}

#[test]
fn m02_day_of_rounds_down() {
    check(day_of(0) == 0, "day_of(0) should be day 0");
    check(day_of(86_399) == 0, "the last second of a day should still be that day");
    check(day_of(86_400) == 1, "86_400 seconds should be day 1");
    check(day_of(1_700_000_000) == 19_675, "day_of(1_700_000_000) should be 19675");
}

#[test]
fn m02_streak_counts_consecutive_days() {
    check(longest_streak(&[1, 2, 3]) == 3, "three consecutive days should be a streak of 3");
    check(longest_streak(&[1, 2, 5, 6, 7, 8, 20]) == 4, "the longest run should win, not the first");
    check(longest_streak(&[7]) == 1, "a single day should be a streak of 1");
}

#[test]
fn m02_streak_ignores_repeats() {
    check(longest_streak(&[3, 4, 4, 5, 9, 10]) == 3, "repeated days should count once and not break a streak");
    check(longest_streak(&[2, 2, 2]) == 1, "the same day repeated should be a streak of 1");
}

#[test]
fn m02_streak_empty_is_zero() {
    check(longest_streak(&[]) == 0, "no days should give a streak of 0");
}
