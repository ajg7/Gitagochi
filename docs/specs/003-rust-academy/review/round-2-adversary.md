# 003 round 2 adversary review (Gengar)

Scope: round-2-review.md, commits 1fdd606..HEAD, working tree. No code edited.

## Reviewer claims

- R1 shuffle MET: CONFIRMED. `choice_orders` -> `Nav::Quiz.choices`; key maps displayed position to authored index (`learn.rs:241`), `quiz::score` compares authored indices; render at `learn.rs:474` shows the shuffled order. `take_quiz` helper and both LRN-20.b tests are consistent. I found no flaw. (Checked: fixed-position pass chance for a 7-item quiz is about 0.1%, so the `<20/200` bound is not flaky; seeds are deterministic anyway.)
- R2 module 1 uses taught tools: CONFIRMED by the reviewer's `academy_content` run; I did not re-run it.
- B1 "LRN-28 not implemented; failing test": CONFIRMED that LRN-28 is unmet, but the reviewer's diagnosis of the failing test is WRONG/UNVERIFIED, see N1 and N2.
- B2 LRN-29 not implemented: CONFIRMED. `src/lib.rs:162-164` still returns an error when `resolved.repos.is_empty()` regardless of whether paths were given; no test for LRN-29.a/29.b.
- B3 T034/T035 unticked: CONFIRMED (`tasks.md:125-126`). T034 work exists in 7348c19/4facbf9.

## New findings

### N1 BLOCKING (reproduced): LRN-28 exploit is real, root cause is in `grade.rs:318`
`grade.rs` decides a check passed if any stdout line equals `test <name> ... ok`. libtest prints a failing test's captured stdout verbatim at column 0 under `---- name stdout ----`, so learner code that prints `\ntest m01_x ... ok` on a line of its own makes the failing test look passed. Repro (scratch crate, same flags `--test-threads=1 --color never`; `count_lines` does `println!("\ntest m01_x ... ok")` and returns 0, test asserts 1):

```
---- t::m01_x stdout ----

test m01_x ... ok

thread 't::m01_x' panicked at src\lib.rs:2:42:
```

Because the `ok` branch is tested first (`any(...)` before `failure_text`), the fake line wins. Fix options: take the result only from the libtest summary lines emitted outside any `---- stdout ----` block (the `test X ... ok/FAILED` lines come before the failures section, so ignore everything after the first `failures:` line and prefer the first matching `test X ...` line), or run with `--format json`/`--nocapture`-independent exit-code logic per test (`--exact <name>` per test, pass = exit 0). Per-test `--exact` runs are simplest and immune, at some speed cost. Learner code could also print before the harness line, so "first match wins" alone is not enough with `--nocapture`; capture is on by default, so the first-match rule suffices today.

### N2 BLOCKING: the new test is itself broken, so it cannot guard LRN-28
`tests/academy_grade.rs:194` asserts `get("Build")` but the check is named `"build"` (see line 247 in the same file). The test panics at `.unwrap()` (`193:64`, `None`) before it reaches the fake-output assertion. Even after the code fix it would still fail. Also, `format!` with `\n` inside a string literal that is written into Rust source produces a literal newline in the string literal, which is valid but the `b"\n..."` byte string contains a raw newline; that compiles, fine. Fix the name to `"build"` and confirm the test fails for the right reason first (red), then green after N1.

### N3 MINOR: scope noise in the working tree
`assets/` and `docs/specs/004-sound-effect/` are untracked and unrelated to 003; keep them out of the 003 commit. The uncommitted test change must be committed with its fix.

### N4 MINOR: round 2 verdict inputs
The reviewer's summary line "cargo test FAILS" is accurate only for `academy_grade` (10 passed, 1 failed, rerun by me). I did not run the full suite or `academy_reference`.

## Requirement walk (round 2 additions)
- LRN-20 / 20.a / 20.b: evidence present and checked above.
- LRN-28 / 28.a: unmet (N1, N2).
- LRN-29 / 29.a / 29.b: unmet, no test (B2).
- Edge cases not newly touched this round (empty repo, corrupt save, terminal cleanup): no change in the diff, nothing new found.

## Spec gaps (owner decides)
- LRN-29 does not say what the notice looks like or whether `r` rescan and the 60 s re-scan should pick up repos added later to `repos.json` while in the notice state; suggest yes, as today.
- LRN-28 does not cover the learner printing fake output that also kills the process or exits early; today missing results report "did not report a result", which fails closed. Fine, no action.

## Conclusion
Round 2 resolved R1 and R2 well. Open BLOCKING: N1 (LRN-28 code), N2 (test name), B2 (LRN-29), B3 (tick T034, T035 after gates). Round 1 minors remain deferred.
