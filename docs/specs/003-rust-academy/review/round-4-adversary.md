# Round 4 adversary (HEAD 1b5f140 vs 312bd07)

Scope: reviewed the diff (grade.rs, lib.rs, academy_grade.rs, spec v1.2) and curriculum/hidden/*.rs. I did not re-run the gates; I rely on the reviewer's run. Findings marked "not reproduced" are from reading the code.

## Reviewer claims
- LRN-28.b met (exit 0 plus nonce marker): CONFIRMED. `grade.rs:357` requires `out.success() && marker.is_file()`. The Drop guard writes only when `!thread::panicking()`. An early `exit(0)` skips Drop, so it fails closed.
- `--exact` name matching nothing leaves no marker, so it fails closed: CONFIRMED.
- "Nonce not passed by env or argv": CONFIRMED, but see M1; the nonce is readable from disk.
- `check_resolved` newline fix: CONFIRMED (`src/lib.rs`, `"\n"`).
- No hidden test uses `should_panic`, `catch_unwind` or a return type (grep over curriculum/hidden), so the "not panicking" rule cannot reject a legitimate pass: CONFIRMED. Hidden tests all have the form `fn name() {`.
- Reviewer's "BLOCKING: none": CONFIRMED. The remaining gaps below are inside the spec's accepted limitation.

## New findings

### MINOR
- M1 (not reproduced). The nonce is easier to find than the known limitation implies. The spec text says cheating needs inspecting the compiled hidden test program. In fact the nonce sits in plain text in places the learner's code can reach with ordinary std calls:
  - `<dest>/tests/academy_mNN.rs` holds the full marker path, and the test runs with cwd = `dest`.
  - `../marks/` is listed with `fs::read_dir`; markers of earlier passing tests are never removed, so while test 2 runs, the file `<nonce>_<test1>` is visible.
  Either lets learner code write `<nonce>_<name>` and then call `exit(0)`, which passes the check. This is a few lines, not binary analysis. Cheap hardening: delete the marker right after reading it (`fs::remove_file` after `is_file()`), and/or write the instrumented tests outside `dest` (they must be inside for cargo, so instead only strip the nonce from the visible source by reading it from a file path outside the tree). Not blocking, because the spec accepts "determined learner", but the known-limitation wording should say "reading the test source or the marks folder" rather than only the compiled program, so the owner decides knowingly.
- M2. A leftover background process spawned by learner code could write a marker after its own test timed out or after the next run starts. The marker is deleted before each test, so a late write only matters within one grade run. Same class as M1.
- M3 (agrees with reviewer minor). `instrument` uses `replacen` on the exact string `fn {n}() {`. A mismatch is silent and fails closed. A content test over `curriculum/hidden` asserting each name appears once in that form would make a curriculum author's mistake loud. The reference test catches it only slowly (about 90s).
- M4. LRN-28.b has one test (module 1, exit(0) in the function). There is no test that a marker is required when the test panics-free but exits non-zero, or that a stale marker from a previous test does not carry over. Low value, optional.

## Spec gaps (owner decides)
- G1. LRN-28 says nothing about a learner reading the grader's files (M1). Decide whether to harden or to widen the known limitation text.
- G2. Open from earlier rounds, unchanged: no end-to-end test for LRN-29.a (reviewer also notes this).

## Edge cases checked
- Windows paths: marker path is embedded with `{:?}` (escaped string literal); the reference-solution test passing on this Windows host (reviewer ran it) shows it works. A data-dir path containing unusual characters is escaped by Debug and stays a valid literal.
- Empty or missing tree, no commits, missing toolchain: unchanged from round 3 and untouched by this commit.
- Timeouts and cancel: marker is removed before each run; `Exit::TimedOut` is handled before the marker check. No regression seen.
- Work folder is removed after grading (`grade.rs:447`), so markers do not leak across runs.

## Recommendation
No blocking items. M1 is the only one worth the owner's attention; it is a small change (remove the marker after reading it) or a one-line spec wording change.
