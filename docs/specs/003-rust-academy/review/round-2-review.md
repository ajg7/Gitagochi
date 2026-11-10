# 003 round 2 review (Jirachi)

Reviewed `4facbf9..HEAD` (commits 1fdd606 spec v1.1, 6701d47 shuffle, 0315076 lessons). Gates run by me: build/clippy/fmt pass; `cargo test` FAILS (see B1).

## Verdict items
- R1 (quiz not guessable): MET. `quiz::choice_orders` + `Nav::Quiz.choices`; answers are mapped to authored indices before scoring (`src/learn.rs:241-243`). Tests `lrn_20_b_no_fixed_position_passes` (200 seeds x 4 positions x every module, <10% pass, each answer lands at more than one position) and `lrn_20_b_retake_reshuffles_and_scores` cover LRN-20.b.
- R2 (module 1 uses only taught tools): MET. `src/academy/content/m01.rs` now teaches `lines()`, `trim().is_empty()`, `let mut` counter and tail expression, matching `curriculum/hidden/m01.rs`. `academy_content` passes (9). m03 borrow-ends-at-last-use and m04 `splitn` added (deferred minors closed).
- R3 (tasks): NOT MET. `tasks.md:125-126` T034 and T035 still unticked.

## Findings

### B1 BLOCKING: LRN-28 not implemented; a failing test is sitting uncommitted
- `tests/academy_grade.rs:193` (working tree, uncommitted, +19 lines): `lrn_28_a_fake_test_output_does_not_pass` fails (`cargo test --test academy_grade`: 10 passed, 1 failed). No change to `src/academy/grade.rs` exists yet, so LRN-28 (spec v1.1, owner ruling S3) is not met. Fix: make grading ignore learner-printed output (for example, run the hidden test binary with results read from a channel the learner code cannot write, or reject results unless a per-run nonce appears), commit the code with the test, and re-run the gates.

### B2 BLOCKING: LRN-29 not implemented
- `src/lib.rs` (run, after `repos::resolve`) is unchanged: with no paths and no repo the app still exits 1. LRN-29/29.a/29.b (and the MR-09 narrowing already written into 002 spec) are unmet; no test exists. Fix: open with a "no repos watched yet" notice when no paths were given; keep exit 1 for explicit paths. Add tests for 29.a and 29.b.

### B3 BLOCKING: T034/T035 boxes (R3 above)
- Tick T034 (done in 7348c19, 4facbf9). T035 only after gates are green and the owner runs, or waives, the manual Windows Verification in plan.md.

### MINOR
- `src/academy/content/m04.rs` (~140): the `splitn` example ends at the iterator; show `parts.next()` / `let a = parts.next()?` so the learner sees how pieces are read.
- Still open from round 1: `checks_upto` tag merge (M2), `probe` ignoring cancel (M3), `p` abandoning a quiz (M4), m09 "characters" vs `.len()` (M6), N2 enclosing-repo export, N3 scaffold retry after `git init` failure, usage text lacks the `l` key.
- Commits 6701d47/0315076 carry the "Opus 5.5" co-author line; harmless.
