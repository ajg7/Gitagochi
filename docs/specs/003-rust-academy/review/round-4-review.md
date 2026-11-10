# Round 4 review (HEAD 1b5f140 vs 312bd07)

Gates run by me: `cargo build` ok, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo test` all pass (academy_grade 12/12, academy_reference 1/1 in 96s). TUI not launched interactively.

## Required item from round 3
- LRN-28 / LRN-28.b: MET. `grade.rs` `instrument()` adds a Drop guard to each hidden test; the guard writes a nonce-named marker outside the learner tree (`<work>/marks/<nonce>_<name>`) only when not panicking. `Grader` passes a check only when exit is 0 and the marker exists (`grade.rs:357`). Early `exit(0)` never runs the Drop, and a `--exact` name that matches nothing leaves no marker, so both fail closed. The nonce is not passed by env or argv. The marker is deleted before each run. `lrn_28_b_early_exit_zero_does_not_pass` covers it; `lrn_28_a_*` stays green. The reference-solution test passing proves the `fn <name>() {` rewrite matches every hidden test.
- Deferred minor (raw newline in `check_resolved`) is fixed in `src/lib.rs`.

## BLOCKING
None.

## MINOR
- `src/academy/grade.rs` `instrument`: if a hidden test is ever written as `fn name()  {` or with a return type, `replacen` silently does nothing and that check always fails with no clear reason. Fails closed, so safe. Optional: a content test in `tests/academy_content.rs` asserting each hidden test name has the exact `fn name() {` form (the reference test already catches it indirectly).
- Nonce is pid plus nanosecond time, so it is guessable by a determined learner. This matches the proposed known limitation; no action.
- Round 1/2 minors and the missing end-to-end LRN-29.a test are still open (see earlier verdicts).

## Spec
Spec v1.2 adds LRN-28.b and the known limitation in the same commit; consistent with the code.
