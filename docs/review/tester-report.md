# Tester report

Result (round-3 pass, commit e31b79e): 50 integration tests across three passes, 0 ignored, 0 failing. cargo test, clippy -D warnings and fmt --check are clean. No BLOCKING bugs. The earlier BLOCKING branch-switch bug is RESOLVED (see Round-3 pass).

## Tests added
- tests/pet_rules.rs (11): hunger rises, hunger cap, health decay only when hungry, feeding bounds, stages, no death (sleepy and recovers), clock backwards, extreme i64 clocks, decay across save/load restart, restart with clock set back.
- tests/persistence.rs (8): missing file, missing parent dir, six corrupt-content variants with .bak kept, binary garbage, directory in place of file, out-of-range clamp, round trip, one file per repo path.
- tests/git_edge.rs (11): non-git dir error text, nonexistent path, subdirectory, empty repo, author filter (other authors, case-insensitive, partial email not matched), regex chars literal, default author from git config, no double feeding across restart, commits made while closed feed on next start, rewritten history, 5000-commit history (baseline only, one new commit fed, under 30s).
All use temp dirs; no real data dir or git config touched.

## Spec clauses covered
Feeding (new commits by configured author only); hunger/health decay; stages; no death in v1; persistence JSON per repo path; missing/corrupt save with .bak; decay from saved timestamps across restart; clock backwards; non-git dir; no commits; large history.

## Not testable (and why)
- TUI drawing, keys q/r, periodic polling, terminal restore on quit/panic/Ctrl-C: ui::run needs a real terminal and has no injectable backend or event seam.
- Default platform data dir (%APPDATA%): store::default_data_dir uses real user dir; not exercised to avoid touching it.
- Windows-only/no-Unix-API: checked only by the suite running on Windows.

## Observations (non-blocking)
- A single poll feeds at most 1000 new commits (MAX_NEW_COMMITS); not stated in the spec.
- Seam suggestion: a testable ui event loop / terminal guard would allow coverage of the terminal-restore clause.

## Round-2 pass: tests/round2.rs (9, 1 ignored)
Covers: branch-switch dedup beyond MAX_RECENT (ignored, BUG), reset to older commit feeds nothing, vanished baseline (feeds nothing, re-baselines, next commit feeds), git failure after .git removal is not a false feed, non-object JSON saves ([] 42 null "str" true) -> fresh pet + .bak, `{}` save -> defaults with last_update=now, partial object keeps present fields, wrong-typed field -> corrupt + .bak, long unicode commit summary truncated to <=200 chars.

## Round-3 pass: tests/round3.rs (11, all passing)
- Branch-switch dedup beyond MAX_RECENT (30 commits) survives save/load restart (no refeed; total_fed unchanged).
- New commit after switching back feeds exactly one.
- fed_ids bounded at MAX_FED_IDS, keeps newest; oversized save trimmed on load without .bak; fed_ids round-trips through save.
- Old saves: no fed_ids with recent -> ids backfilled, HEAD not refed, next commit fed; minimal-field save loads (no .bak) with empty fed_ids.
- Vanished baseline: garbage id and gc-pruned commit both re-baseline, feed nothing, return Ok, next commit feeds.
- Summaries: ESC[..m / ESC[2J / ESC[1;1H sequences and BEL/BS/DEL stripped (commit made via -F with verbatim cleanup); plain and unicode summaries untouched.

## Bugs
No BLOCKING bugs.
- RESOLVED: double feeding on branch switch. tests/round2.rs `switching_to_branch_with_many_fed_commits_does_not_refeed` is no longer #[ignore]d and passes; tests/git_switch.rs and tests/round3.rs also cover it.
- Known limitation (gatekeeper deferred item 2, spec ambiguous, not asserted): never-seen older commits on another branch are fed on first checkout.
- Not covered: OSC text and bidi override characters in summaries (deferred item 5) are known to survive strip_control; not asserted.

## Environment note
A stale target/debug/gitagochi.exe lock (Access denied) blocked `cargo test` in the default target dir; verification was run with CARGO_TARGET_DIR set elsewhere.
