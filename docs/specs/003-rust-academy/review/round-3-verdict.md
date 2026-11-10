VERDICT: REVISE
ESCALATE: yes

Gates run by me (cargo from ~/.cargo/bin): `cargo build` ok, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo test` all binaries pass (including academy_reference, 88s). Tasks: all ticked except T035 (manual Windows check, pending by design). I did not launch the TUI interactively (no terminal in this run); LRN-29 is covered by unit tests only.

## Required (BLOCKING)
1. LRN-28, `src/academy/grade.rs:324` (`else if out.success() { Ok(()) }`): a check passes on exit code 0 alone. Learner code that calls `std::process::exit(0)` inside the function under test ends the hidden test binary with code 0 before `check()` runs, so the check passes. This violates the first clause of LRN-28 ("result shall depend only on whether the checks really passed"). It also passes when `--exact <name>` matches no test (0 tests run, exit 0). Round 2 failed closed; this is a regression.
   Expected outcome: pass only when exit is 0 AND there is positive evidence that the test completed. Do not pass the evidence through env or argv (the learner process can read those). Suggested: the hidden `check()`/test end writes a success marker containing a nonce that is baked into the hidden source as a const when it is exported (not present in the learner tree), and grading requires exit 0 plus that marker for exactly this test; or equivalent. Add an integration test in `tests/academy_grade.rs` (name `lrn_28_*`): a committed project whose function under test calls `std::process::exit(0)` must fail that check. Existing `lrn_28_a_fake_test_output_does_not_pass` must stay green.

## Deferred (minor)
- `src/lib.rs:~160-164`: `check_resolved` joins with a literal raw newline inside a string; write `"\n"`.
- No end-to-end test for LRN-29.a (no paths, no repo, `l` opens learn); pieces are unit tested.
- Child stdin may not be null for hidden test runs; bounded by the test timeout. Unverified.
- Round 1/2 minors still open, as listed in the round-2 verdict.

## Proposed known limitations
- "Grading cannot stop a determined learner from cheating by inspecting the compiled test program (for example reading its executable to find the success marker). This is a local self-study tool; only accidental or output-based false passes are prevented."

## Spec questions for the owner
- Tighten LRN-28 to "nothing the learner's code does (printing, exiting early, panicking in another thread) shall make a check pass"? The current acceptance criterion LRN-28.a only covers printed fake lines; add an acceptance for early `exit(0)`.
- LRN-29 does not state the notice wording or location; acceptable as is.
