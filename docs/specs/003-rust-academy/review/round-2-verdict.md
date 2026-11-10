VERDICT: REVISE

Round 2 (Alakazam). Checked: `cargo build` OK, `cargo clippy --all-targets -- -D warnings` OK, `cargo fmt --check` OK, `cargo test --test academy_grade` FAILS (10 passed, 1 failed: `lrn_28_a_fake_test_output_does_not_pass`). Reviewer and adversary agree on all blocking items; I confirmed each in the code. R1 (quiz shuffle, LRN-20.b) and R2 (module 1 content) are met. I did not launch the interactive TUI; no source relevant to startup changed this round except the unmet LRN-29 below.

## Required (BLOCKING)

1. LRN-28 / LRN-28.a: `src/academy/grade.rs:~318`. A check is marked passed if any stdout line equals `test <name> ... ok`, and the `ok` branch is tested before `failure_text`. Learner code can print `\ntest m01_x ... ok` and so fake a pass, because libtest echoes a failing test's captured stdout at column 0. Expected outcome: results come only from libtest's own summary lines, not from anything the learner prints. Either read only the `test X ...` lines before the first `failures:` line (first match wins; capture is on by default), or run each test with `--exact <name>` and use the exit code. A test whose result is missing must still fail closed. Commit the fix together with the test.
2. LRN-28 test: `tests/academy_grade.rs:~194`. The test uses `get("Build")` but the check is named `"build"` (see ~247). It panics on `unwrap()` before reaching the fake-output assertion. Expected outcome: fix the name, confirm the test fails because of the exploit (red), then passes after item 1 (green). Commit it (currently uncommitted).
3. LRN-29 / 29.a / 29.b: `src/lib.rs:~162-164` (`run`, after `repos::resolve`). With no paths given and no repo found, the app still returns an error and exits 1. Expected outcome: when no paths were given and nothing is watched, open the TUI with a "no repos watched yet" notice (the academy still works). Explicit paths that resolve to nothing keep the error and exit 1. Add tests for 29.a and 29.b. 002's MR-09 narrowing is already in its spec.
4. Tasks: `docs/specs/003-rust-academy/tasks.md:125-126`. Tick T034 (done in 7348c19 and 4facbf9). Tick T035 only after all gates are green and the owner has run, or waived, the manual Windows Verification in `plan.md`.

Keep `assets/` and `docs/specs/004-sound-effect/` out of the 003 commit.

## Deferred (minor)
- `src/academy/content/m04.rs` (~140): `splitn` example should show reading pieces (`parts.next()`).
- Round 1 minors still open: `checks_upto` tag merge, `probe` ignoring cancel, `p` abandoning a quiz, m09 "characters" vs `.len()`, enclosing-repo export, scaffold retry after `git init` failure, usage text lacks the `l` key.
- Commit co-author lines on 6701d47 and 0315076 say "Opus 5.5". Harmless.

## Proposed known limitations
- None this round. (If wanted: "Grading reads results from libtest summary lines only; a learner program that aborts the process reports 'did not report a result'.")

## Spec questions for the owner
- LRN-29: what should the no-repos notice say, and should `r` and the 60 s re-scan pick up repos added to `repos.json` while in that state? Adversary suggests yes, as today. Needs the owner's decision or a spec line.
- LRN-28: learner output that kills the process or exits early already fails closed ("did not report a result"). No action unless the owner wants it written down.
