# Round 3 adversary (Gengar)

Method: read the round-3 review, `src/academy/grade.rs` (`tests`, lines 257-333), `src/lib.rs` (`check_resolved`), `curriculum/hidden/m01-m03.rs`, `tests/academy_content.rs`, spec LRN-28/29. I did not run new code; the exploit follows directly from the code (labeled below).

## Reviewer claims
1. BLOCKING exit(0) hole in LRN-28: CONFIRMED. `grade.rs:324` passes on `out.success()` alone. The hidden tests link the learner's library in the same process (`check(count_lines(..) == 3, ..)` in `curriculum/hidden/m01.rs`), so a learner `count_lines` that calls `std::process::exit(0)` ends the test binary with code 0 before `check` runs, and the check passes. Reasoned, not executed. This breaks LRN-28 ("nothing the learner's program prints shall make a check pass"; the spirit is that only a real pass counts) and is a regression from round 2, which failed closed. The `lrn_28_a` test only covers printed fake lines, which is why it is green. Severity BLOCKING is right.
2. `--exact <name>` matching no test gives exit 0: CONFIRMED as libtest behavior, but OVERSTATED as a risk. `lrn_17_hidden_tests_match_checks` (`tests/academy_content.rs:115-141`) asserts every curriculum check name appears as `fn <name>(` in the hidden file, and `academy_reference` runs each solution, so a mismatch is caught in CI. The weakness is only a runtime fail-open if someone breaks that alignment; the nonce/evidence fix covers it too.
3. Reviewer's proposed fix: the nonce idea is sound but is not enough by itself. `check()` runs inside the learner-reachable process, but the nonce can only be forged if the learner can read it. If it is generated per run and passed in via env var to the test binary, learner code can read `std::env::var` and print the line, then `exit(0)`. Do not pass the nonce by env or argv. Safer options: (a) have `check()` print the success marker only at the very end of each test fn (a final `check_done()` call), and require exit 0 plus that marker, using a per-run nonce that is baked into the hidden source at export time (a `const`, written when the hidden file is copied, not present in the learner's tree). The learner's library cannot name a const in the test crate, but it could still scan the process's own executable or `/proc/self/exe` for it, which is far out of scope for a teaching app. (b) Cheaper: write the result to a file whose path is in the hidden file only (same caveat). State the residual risk (a determined learner can cheat; this is a local self-study tool) in Known limitations rather than chasing it. Note also that `std::process::abort` gives a nonzero exit and fails closed, as does a panic; only exit(0)-style paths matter.
4. Test for exit(0): the reviewer asks for one. Agree; it must be added as a new `lrn_28_*` integration test in `tests/academy_grade.rs`. `rg process::exit tests/academy_grade.rs` shows none exists.
5. LRN-29/29.a/29.b: CONFIRMED met in code via `check_resolved` and `ui.rs` notice. No end-to-end test for 29.a; MINOR, agree.
6. T034 ticked, T035 unticked: no objection.

## New findings
- MINOR: `src/lib.rs:~160-164`: reviewer is right that `join("` + raw newline + `")` is a literal newline in a string. It compiles and behaves, rustfmt accepts it, but it reads badly and is easy to break; use `"\n"`. CONFIRMED.
- MINOR (not in the review): stdin is not obviously closed for the hidden test child; a learner function that reads stdin could hang until the test timeout. This is bounded by `test_timeout`, which `tests()` handles via `left`. Unverified that stdin is null in `self.run`; low value.
- MINOR (unverified): when the `--exact` run is judged only by exit code, a learner `main`-less panic inside a spawned thread that the test does not join could exit 0 with the test passing when the test itself did not fail. That is the correct outcome for libtest, so not a bug.

## Spec gaps (owner decides)
- LRN-28 says "nothing the learner's program prints shall make a check pass" but is silent on a learner program that exits early. Suggest tightening to "nothing the learner's code does (printing, exiting, panicking in another thread) shall make a check pass", and listing the residual attack (reading the test executable) under Known limitations.
- LRN-29 does not say what the notice says or where it appears; acceptance 29.a only requires "shows the notice". Fine, but untestable beyond the unit tests.

## Edge cases checked (no new problems)
- Empty repo / no commits: `grade` returns `NoCommits` (`grade.rs:387`). Non-git dir: `NotARepo`. Missing folder: `FolderMissing`. Fine.
- Timeouts: per-run `left` budget with `saturating_sub`; fine.
- Windows paths: `exe` comes from cargo JSON `executable`, used via `to_string_lossy`; no issue seen.
- Not checked: corrupt progress file and terminal cleanup on panic in this round (unchanged from round 2).

## Verdict input
One BLOCKING item stands: LRN-28 fails open on early `exit(0)` (and, in principle, on a zero-match `--exact`). Required: require positive evidence of a completed test in addition to exit 0, add a test with a project that calls `std::process::exit(0)` in the function under test and assert that check fails, and record the residual limitation in the spec. Everything else is MINOR.
