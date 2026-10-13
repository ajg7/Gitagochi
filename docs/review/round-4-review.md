# Round 4 review: multi-repo (commits e9046f1, 69ec874)

Checks run by me: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test` all pass (no failures in any test binary).

## BLOCKING

1. README repos.json example is invalid JSON. README.md Usage section (the `{"repos": ["C:\Users\me\Tech & Coding", ...]}` line).
   - `\U` is not a valid JSON escape. `serde_json` rejects it in `repos::parse_config` (src/repos.rs:29-31), so a user who copies the example gets "invalid repos.json: invalid escape ...".
   - What the user then sees (src/repos.rs:35-40, src/lib.rs `run`, src/ui.rs `refresh`):
     - `load_config` falls back to the default config and returns a warning.
     - With no CLI paths the app silently falls back to watching `.`.
     - If `.` is a repo, the app runs on the wrong repo. The only sign is a one-line "warning: invalid repos.json..." in the message area.
     - That warning is shown only when there are no sync errors (`refresh` uses `else if`).
     - The warning is also dropped at the first rescan (about 60s, or on `r`), because src/ui.rs does `warnings = r.warnings;` and discards `session.initial_warning`.
     - If `.` is not a repo, the user gets a clear message and exit 1, which is acceptable.
   - Fix:
     - Change the README to escaped backslashes (`"C:\\Users\\me\\Tech & Coding"`) or forward slashes (`"C:/Users/me/..."`, which work on Windows). Mention that raw backslashes must be doubled.
     - Make an unparseable repos.json a fatal error (exit 1) when it would otherwise be used, i.e. when no CLI paths were given. At minimum keep the config warning permanently in the UI. Include the line/column from serde in the message. A broken config silently turning into "feed from cwd" is wrong behavior.
     - Add a unit test that feeds a single-backslash path to `parse_config` and checks the error.

## MINOR (some significant)

2. Polling cost on the UI thread. src/ui.rs `refresh` calls `sync_all` synchronously, once per 10s.
   - Each idle repo costs one `git rev-parse` process (`Repo::head`). Changed repos cost 2 more (the baseline `rev-parse` and `log`). On Windows, process spawn is about 30-60ms, so 50 repos block the loop for roughly 2-3s every 10s.
   - During that time input (`q`) and redraw are unresponsive. `resolve` (the folder scan) also runs on the UI thread every 60s.
   - Suggested fix:
     - Do polling and scanning on a worker thread that sends results over an mpsc channel.
     - Or at least scale the poll interval with repo count.
     - Or read `.git/HEAD` and the ref file directly to skip the spawn when nothing changed.
   - The 60s rescan interval satisfies the spec ("at least every minute").

3. Scan only checks the name for hidden dirs (src/repos.rs:73-75). Windows hidden/system dirs without a leading dot (for example `AppData`, `$Recycle.Bin`) are scanned if someone passes a broad parent like the user profile. It is bounded by depth 3, but it is slow and runs on the UI thread. Consider also skipping directories with the HIDDEN (0x2) or SYSTEM (0x4) attribute on Windows.

4. De-dup of repos uses `key().to_lowercase()` (src/repos.rs:109). On a case-sensitive filesystem two different repos could collapse. This is fine on Windows, but the save key itself is case-sensitive, so differing-case paths could still produce two tracking entries for one repo if `canonicalize` fails and falls back to the raw path. Low risk.

5. `resolve` for a path that is a subfolder of a repo (src/repos.rs:122-129). If the subfolder contains nested repos (for example a monorepo `tools/` holding other repos), `scan` finds those and the containing repo is never watched. This is an edge case. Legacy saves keyed by the containing repo would then not carry over. Consider trying `Repo::open` first when `.git` is not found upward.

6. Migration (src/store.rs `migrate_legacy`) is correct for the main path:
   - The legacy key is matched through `save_path(data_dir, key)`.
   - Tracking and `recent` ids are carried over, so there is no pet reset and no refeed.
   - The newest file wins, and it is skipped if `pet.json` or `pet.json.bak` exists.
   - One limitation to document: tracking of the other legacy repos, and of the winner if it is not in the current selection, is dropped. Those repos will re-baseline when first seen, so commits made in the gap are not fed. This is acceptable under the spec, but it is worth a README line.

7. Removed or changed test coverage:
   - Removed: the `old_save_without_fed_ids_*` tests, in tests/git_switch.rs and tests/round3.rs. These were the old-save dedup of `recent` for the HEAD commit. The behaviour now lives in `migrate_legacy` (src/store.rs, the `recent` loop). tests/multi_repo.rs `legacy_save_is_adopted_with_tracking_and_does_not_refeed` covers it, so coverage is retained.
   - Weakened: `fed_ids_are_bounded_and_keep_newest` (tests/round3.rs) used to assert through `Pet::feed` that `total_fed` counts all fed commits and `recent` is capped at `MAX_RECENT` for a large batch. It now tests only `RepoTrack::record`. Add back an assertion that feeding more than `MAX_RECENT` commits keeps `recent.len() == MAX_RECENT` and `total_fed` correct. The other changed tests only adapted field access and still cover the original behaviour.

8. `ui::refresh` shows only the first error or warning (src/ui.rs). With several failing repos the user sees only one. This is fine for a "one-line warning", but the count could be shown ("+2 more").

9. Cosmetic: `".git"` in `SKIP_DIRS` (src/repos.rs:19) is redundant, because `skipped` already drops names starting with `.`. In src/lib.rs `run` the `msgs.join("\n")` is written as a literal newline inside the string, which is odd formatting. Use `"\n"`.

## Confirmed fine

- 3-level scan depth, no descent into repos, skip list, and the junction/symlink skip (reparse-point check on Windows).
- Per-repo `RepoTrack` with its own baseline, re-baseline on a vanished baseline, and per-repo error isolation in `sync_all`.
- Shared `pet.json`.
- UI repo count and `[repo]` tags.
