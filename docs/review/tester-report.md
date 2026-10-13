# Tester report

Latest: final pass (commit 1692984) at the end of this file. 0 BLOCKING bugs; the earlier multi-repo BLOCKING bug is RESOLVED.

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

## Multi-repo pass (v1.1, after round-5 SATISFIED): tests/multi_repo_spec.rs
28 tests (27 pass, 1 ignored for a BLOCKING bug). All use temp dirs; the real data dir is never written. Two binary tests pass CLI paths, so `run()` only reads the config and exits before touching any save.

Covered: selection precedence (CLI over config repos, config over current dir, `.` fallback), `--author` and `--author=` parsing, config supplying repos and author, author filter applied per repo (and an overriding author), scan depth 1-3 (depth 4 excluded), skip list (node_modules, target, venv, .venv, __pycache__, hidden dirs), no descent into repos, silent skip of plain dirs, duplicate paths counted once, junctions not followed (Windows), first-seen repo baselines only (20-commit history), baseline vanished re-baselines only that repo, one shared pet and one save file across three different selections, missing and non-repo paths warn while others feed, no resolvable repo gives empty result, legacy migration (commits made while closed feed only the new ones, no reset; pet kept when its repo is not selected and the new repo only baselines; shared save blocks migration and leaves the legacy file; corrupt legacy save skipped), repos.json with BOM, doubled backslashes, single backslashes (clear error), unparseable or empty file gives a warning from load_config, unknown/partial keys, binary exits 1 with an `error:` message when no repo resolves (missing paths, plain dir).

Not testable: the fatal "unparseable repos.json and no CLI paths" exit lives in `run()`, which reads the real platform data dir (`dirs::data_dir()` ignores env overrides on Windows) and cannot be redirected. Only its building block (`load_config` warning) is tested. Seam suggestion: let `run` take a data dir, or honour a `GITAGOCHI_DATA_DIR` env var. Also untestable: UI repo labels and the "N repos watched" header, the periodic rescan timer, the in-UI warning line (no terminal seam).

### BLOCKING
- **Repo that is temporarily unreadable loses its baseline and refeeds its history.** Test: tests/multi_repo_spec.rs:329 `temporarily_unavailable_repo_keeps_baseline` (ignored). Spec: "A repo seen for the first time only baselines its HEAD (existing history does not feed)" and "only walk new commits". Cause: src/lib.rs `sync_repo` sets `track.last_seen = poll.head`, and `Repo::commits_since` returns `head: None` when `rev-parse HEAD` fails (repo dir renamed, drive unmounted, .git briefly unreadable) with no error. last_seen becomes None while `initialized` stays true, so on return the whole history (up to 1000 commits per repo) feeds. Expected: keep the old last_seen when head is None. Repro: `cargo test --test multi_repo_spec -- --ignored temporarily_unavailable` (fails with left 2, right 1: base commit refed plus the new one).

### Observations (non-blocking)
- A vanished repo mid-run produces no warning or error (head None is treated as success); the user sees nothing until the next rescan.
- Reset to an older commit then a new commit: the new commit feeds (old baseline still exists in the object db). Consistent with the known branch-switch limitation; not asserted.
- Windows: stale target lock issue persists; verification used CARGO_TARGET_DIR in the temp dir.

## Final pass (commit 1692984, after round-6 SATISFIED): tests/repo_unavailable.rs
Status: the previously BLOCKING `temporarily_unavailable_repo_keeps_baseline` (tests/multi_repo_spec.rs:328) is RESOLVED: no longer ignored, passes. Full suite: 0 failed, 0 ignored. cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean. Temp dirs only; the real data dir is untouched. No new BLOCKING bugs.

6 new tests (all pass):
- warning_appears_for_repo_with_baseline_whose_head_is_unreadable: exactly one "b: repo unavailable" error, healthy repo a still feeds.
- no_warning_for_empty_repo: no warning over three polls, nothing fed.
- no_warning_for_freshly_added_repo: repo added after the first sync only baselines, no warning, nothing fed.
- recovery_does_not_refeed_and_new_commits_feed: repo with 6 commits hidden for 3 polls (warning each poll, nothing fed); on return nothing refed, warning clears, 2 new commits feed.
- commits_made_while_unavailable_feed_once_on_recovery: the one commit made while away feeds once, then 0.
- baseline_survives_save_load_while_unavailable: baseline kept across save/load while hidden; recovery refeeds nothing, next commit feeds.

Known limitation (recorded, NOT blocking, per gatekeeper): a repo unreadable at its very first sync is stored initialized with last_seen=None, so on recovery its existing history refeeds (up to 1000 commits). Not asserted by any test. Spec clause (docs/spec.md:34) is met whenever HEAD is readable on first sight.

Not testable: the in-UI display of the warning (ui.rs shows only `errors.first()`, so it can hide other messages); no terminal seam. The spec only says "one-line warning in the UI".
