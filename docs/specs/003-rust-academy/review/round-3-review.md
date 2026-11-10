# Round 3 review (Jirachi)

Gates run by me: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` (all binaries, including academy_reference): all pass. `lrn_28_a_fake_test_output_does_not_pass` is now green.

## Required items from round-2 verdict
1. LRN-28 / 28.a: partly met. The printed-line exploit is closed (each test runs with `--exact`, judged by exit code; `src/academy/grade.rs:~307-331`). New hole below.
2. LRN-28 test name fixed ("build") and committed: met.
3. LRN-29 / 29.a / 29.b: met in code. `check_resolved` (`src/lib.rs:~149`) keeps the error for explicit paths and opens with no paths; `src/ui.rs:~334` shows the notice; unit tests cover the notice and the explicit-folder error. No end-to-end test for 29.a (`l` opens the learn screen with no repos), but the pieces are tested, so MINOR.
4. Tasks: T034 ticked. T035 correctly left unticked (manual Windows check pending).

## BLOCKING
- `src/academy/grade.rs:~322-325` (LRN-28): grading now treats exit code 0 as a pass. Learner code that calls `std::process::exit(0)` inside the function under test (or any path that exits 0 before the assertion) makes that check pass. Round 2 failed closed ("did not report a result"); this is a regression. It also fails open when `--exact <name>` matches no test (0 tests run, exit 0), so a name mismatch between curriculum and hidden file would silently pass. Fix: require exit 0 AND positive evidence that exactly this test ran. For example, have the hidden `check()` helper print a per-run random nonce line on success (the learner cannot read the hidden source or the nonce), and pass only when exit is 0 and the nonce appears. A cheaper partial fix is to also require stdout to contain `test <n> ... ok` and `1 passed`, but learner output can still forge that, so the nonce is the real fix. Add a test: project whose function calls `std::process::exit(0)` must fail that check.

## MINOR
- `src/lib.rs:~160-164`: `check_resolved` uses a string literal with a raw newline in `join("\n")`; write `"\n"`.
- No integration test that no-paths plus no repo reaches the TUI (29.a); acceptable given unit coverage.
- Deferred round 1/2 minors still open, as listed in the round-2 verdict.
