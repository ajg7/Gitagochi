VERDICT: SATISFIED

ESCALATE: no

## Evidence
- Gates run by me at HEAD 1b5f140: `cargo build` ok, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo test` all suites pass (academy_grade 12/12, academy_reference 1/1 in about 102s).
- Round 3 required item (LRN-28 / LRN-28.b) is met: a check passes only with exit 0 plus the nonce marker (`grade.rs:357`); early `exit(0)` and a non-matching `--exact` name fail closed. Reviewer and adversary agree, and I found no disagreement to resolve.
- Spec v1.2 matches the code. No BLOCKING items from reviewer or adversary.
- T035 is the only unticked task. Its gates half is done by this run. The manual TUI check on Windows (plan.md Verification) was not done by me: I did not launch the TUI interactively. The owner or tester should do it before closing.

## Required items
None.

## Deferred (minor)
- `instrument` in `src/academy/grade.rs`: a hidden test not written as exactly `fn name() {` makes `replacen` do nothing, so that check always fails. It fails closed. Add a content test in `tests/academy_content.rs` over `curriculum/hidden`.
- Optional hardening: delete the marker right after reading it. It would not stop a learner who writes the marker for a later test, but it narrows the window.
- Leftover learner background process could write a marker late. It matters only within one grade run (adversary M2).
- Missing tests for a stale marker not carrying over, and for non-zero exit with a marker (M4).
- Missing end-to-end test for LRN-29.a. Round 1 and 2 minors are still open.

## Proposed known limitations
- LRN-28.b: grading resists fake output but not a determined learner. The nonce marker can be found by reading the instrumented test source in the project's `tests/` folder (it holds the marker path), by listing the `marks` folder (markers of earlier passing tests stay until the run ends), or by inspecting the compiled test program. The nonce is derived from the process id and the clock. Learner code that writes the marker and then exits 0 would pass that check. The grader is a learning aid, not a security boundary.

## Spec questions for the owner
- G1: LRN-28 does not say whether a learner reading the grader's files is in scope. Widen the known limitation (above) or harden by removing markers after reading them.
- G2: should LRN-29.a get an end-to-end test?
