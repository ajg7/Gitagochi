# Round 3 adversary (commit e31b79e)

## Verification of reviewer claims
- "No BLOCKING; branch-switch refeed fixed": CONFIRMED for the scenario in tests/git_switch.rs (feat -> main -> feat). I re-read sync() and feed(); dedup via fed_ids is correct. I did not re-run the full suite.
- Minor 1 (400 KB save): UNDERSTATED. `refresh()` in src/ui.rs sets `dirty = true` on every poll (every 10 s, and on every error), not only when something changed. With a full 10,000-id set and `to_string_pretty` (one id per line, about 50 bytes each) that is a ~500 KB write plus rename every 10 s indefinitely. Still MINOR, but the fix is trivial (set dirty only when n > 0 or last_seen/initialized changed).
- Minor 2 (`has_fed` unused by sync): CONFIRMED. The reviewer calls the function `normalize`; it is actually `Pet::sanitize`.
- Minor 3 (OSC leftovers): CONFIRMED, cosmetic. Also not handled: bidi overrides (U+202E, category Cf, not `is_control`). Cosmetic.
- Minor 4 (rev-parse warnings become persistent GitError): OVERSTATED. `--verify --quiet` on a missing object prints nothing and exits 1, so the vanished-baseline path works. Only stray git warnings would trigger it, which is hypothetical and unverified. Also: when the baseline has vanished and the user commits before the next poll, that commit is silently dropped (re-baselines to HEAD). MINOR and by design.
- Minor 5, 6: CONFIRMED as stated.

## New findings
### MINOR A (reproduced): never-seen older commits on another branch feed the pet
The baseline run records only `last_seen`, not the existing history in `fed_ids`. Checking out a branch that has pre-existing, never-fed commits by the author makes `last_seen..HEAD` include them, and they are fed (up to 1000 per poll).
Repro (temporary integration test, since deleted): init repo, commit `base` on main; create branch `old` with 5 empty commits; back on main run the first `sync` (baseline); `checkout old`; `sync` returned 5 (`FED 5`, assertion `left: 5 right: 0` failed).
Spec: "each new commit ... since the last seen commit". These are not new commits, so this is arguably a spec gap and the same class as the refeed bug just fixed. Not blocking, because the spec wording is ambiguous, but a user can farm hunger relief by switching to old long-lived branches. Possible fix: use `git rev-list HEAD --not --all`-style baseline, or record fed state by "commit time >= pet creation time".

### MINOR B: old saves lose dedup beyond 20 commits
For saves without `fed_ids`, only the 20 ids in `recent` are backfilled. Branch-switch refeeding of older commits remains possible for migrated pets. One-time, low impact.

### MINOR C: no cap on feed per poll beyond 1000, and feed uses the newest 1000 only
With `--max-count=1000`, anything older in a huge range is skipped, and `last_seen` still jumps to HEAD. Acceptable under "only walk new commits"; noted for completeness.

## Checked, no issue found
- Terminal restore: `ratatui::init` installs the panic hook, and `q`, Esc and Ctrl-C (raw mode key event) break out to `ratatui::restore()`. An early `?` in the loop still reaches `restore()`, because `run` calls it after `event_loop` returns. A non-key SIGINT or console close is not handled (Windows Ctrl-Break); unverified and outside the spec's literal wording.
- Corrupt or missing save, clock going backwards, non-git dir, empty repo: handled by existing code and unit tests; nothing new in this commit.
- Windows paths: `save_path` lowercases the key unconditionally in the current code (the reviewer's note about `cfg!(windows)` is stale or the item is unaddressed, either way harmless on Windows).

Verdict suggestion: no BLOCKING issues found; MINOR A is the most worthwhile follow-up.
