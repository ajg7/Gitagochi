# Round 4 adversary: multi-repo (commits e9046f1, 69ec874)

Method: read the spec "Multi-repo (v1.1)" section, src/repos.rs, store.rs, lib.rs, ui.rs and git.rs, and checked them against the reviewer's claims. I also inspected the user's real data dir (`%APPDATA%\gitagochi`) and ran two small experiments (below). I did not rerun cargo build, test, clippy or fmt; I take the reviewer's word that they pass.

## Verdicts on reviewer claims

1. README repos.json example is invalid JSON: CONFIRMED.
   - `\U` is not a valid JSON escape, and the spec itself uses `\\`.
   - The fix is trivial, so BLOCKING is fair given the user is on Windows and will copy it.
   - The "make it fatal" suggestion is a design opinion. The spec only says a bad config path warns; it does not cover an unparseable config. The reviewer's claim that the warning is dropped at the first rescan is CONFIRMED: src/ui.rs `warnings = r.warnings;`.
2. Polling cost: OVERSTATED for the user's layout.
   - 12 repos means 12 `rev-parse` calls per 10s poll. I timed 12 sequential `git rev-parse` runs at about 1.2s inside Git Bash, which inflates the cost with fork overhead. Native `Command` spawn is likely 0.3-0.7s.
   - That is a noticeable input stall every 10s, not the "2-3s for 50 repos" scenario. MINOR stands, but note that `q` can lag by up to roughly half a second at the user's real size.
3. Hidden/system dirs on Windows: CONFIRMED, MINOR. It only matters for broad parents, and the user's parents are not broad.
4. Case-insensitive de-dup: CONFIRMED, harmless on Windows.
5. Subfolder of a repo that contains nested repos: CONFIRMED, edge case.
6. Migration: CONFIRMED correct. I verified against the user's real files.
   - The legacy file `Gitagochi-e12b1a14782de6e5.json` exists, and `pet.json` already exists (it was created by a dev run at 22:47).
   - pet.json's `repos` key is `\\?\C:\Users\ajgeb\Desktop\Tech & Coding\Coding_Project\Gitagochi`, so the canonical key matched the legacy hash and tracking carried over.
   - Consequence for the user: migration is one-shot and already done. Deleting pet.json would re-migrate from the legacy file; deleting it AND the legacy file would reset the pet.
7. Test coverage changes: CONFIRMED. The `old_save_without_fed_ids_*` tests are gone (grep finds none), and `fed_ids_are_bounded_and_keep_newest` is in tests/round3.rs.
8. and 9. Cosmetic: CONFIRMED.

## New findings

N1. MINOR, Windows-specific and related to the BLOCKING item: a UTF-8 BOM in repos.json makes the config unparseable.
- Windows PowerShell 5 (`Set-Content -Encoding UTF8`, `Out-File`) and some Notepad versions write a BOM, which `serde_json` rejects.
- Reproduction, with a scratch crate calling `serde_json::from_str("\u{feff}{\"repos\":[]}")`: `Err("expected value at line 1 column 1")`.
- `load_config` then falls back to the default config. With no CLI paths the app silently watches `.`.
- This is the same silent-fallback failure as the reviewer's item 1. Fix: strip a leading `\u{feff}` in `parse_config`, and add a unit test.

N2. MINOR (design gap): one author is applied to all repos.
- src/lib.rs `run` takes the first repo (sorted scan order) that has `user.email`, and uses it for every repo.
- For a user with per-repo `user.email` overrides (for example work vs personal, or Game Development vs Tech & Coding), commits in repos with a different email are silently never fed.
- The result also depends on which repo sorts first, so the effective author can change when repos are added.
- Spec: "configured author (default: git config user.email)" is ambiguous here.
- Mitigation: document that `author` in repos.json should be set, or default per repo.

N3. MINOR: a re-added repo can feed a backlog.
- Tracking entries are never removed from `pet.repos` when a repo disappears from the selection.
- If a repo is dropped and re-added later, its old `last_seen` is reused, so all of that repo's commits made in the gap feed the pet (up to 1000).
- The spec says "a repo added or removed between runs" but does not say which behaviour is wanted. This is plausible and arguably desirable, but it is undocumented and untested.
- The reviewer's item 6 says dropped repos "re-baseline when first seen". That is true only for repos that were never tracked. Stale entries are also never pruned, but they are small.

N4. MINOR: pet.json is re-saved every poll (10s) as pretty-printed JSON, even when nothing changed. `fed_ids` can reach 10,000 ids per repo, about 400 KB per repo. This is realistic only for long-time heavy committers, and it mostly costs disk writes and UI-thread time.

N5. MINOR: the `[repo]` tag uses only the folder name. Two repos with the same folder name under different parents (for example `Tech & Coding/x/app` and `Game Development/app`) look identical in the UI. Tracking is still keyed correctly by full path.

N6. MINOR: `repos.json` is read once at startup. Edits during a session are not picked up. The spec only requires re-scan of folders, so this is fine, but it could be documented.

## Checked and found fine

- Scan depth: repos at levels 1-3 are found and a level-4 repo is not (tested in `scans_three_levels_and_skips_junk`).
- Names with spaces, `&` and hyphens: `Command::arg` passes them without a shell. `save_path` sanitizes the name for the legacy file. The `&` appears in the real migrated key and works.
- Repos with no commits: `commits_since` returns early while HEAD is None. The first commit later feeds, since the baseline is None.
- Corrupt or missing pet.json: `load` backs it up and starts fresh. `migrate_legacy` is skipped if pet.json or pet.json.bak exists.
- Terminal cleanup on panic and Ctrl-C: unchanged from earlier rounds. Ctrl-C is handled as a key event in raw mode.

## Overall

The reviewer's findings hold up. Only the item 2 magnitude was overstated. Nothing new reaches BLOCKING beyond the reviewer's item 1. I recommend folding N1 into that same fix (strip the BOM, fix the README, and keep the config warning visible).
