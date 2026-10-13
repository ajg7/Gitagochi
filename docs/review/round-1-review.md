# Round 1 review

Checks run by reviewer: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` (16 unit + 3 integration) all pass. `--help` and non-git-dir error path verified. The interactive TUI was not exercised.

No BLOCKING findings. Spec conformance is good: baseline-on-first-run, only-new-commit walk (`since..HEAD`, capped at 1000), .bak on corrupt save, clock-backwards handled, terminal restore via `ratatui::init` panic hook plus Ctrl-C handled as a key in raw mode.

## MINOR

1. src/ui.rs:113 `&c.id[..c.id.len().min(7)]` slices by byte. A hand-edited save with a non-ASCII `id` panics at a non-char boundary during draw. Fix: `c.id.chars().take(7).collect::<String>()`.
2. src/git.rs:114-115 Switching branches or checking out an unrelated commit makes `last_seen..HEAD` include already-fed (or old) commits, which feed again, up to 1000. There is no dedupe against `pet.recent` or a seen set. Fix: use `merge-base`, or skip ids present in `recent`, or only advance `last_seen` without feeding when `since` is not an ancestor of HEAD (`git merge-base --is-ancestor`).
3. src/git.rs:73-75 / 133-139 Any `git rev-parse` failure (e.g. "dubious ownership") is reported as "not a git repository", and any `git log` failure is silently treated as a rebase and re-baselined, which hides real errors. Fix: surface stderr for the non-"not a git repository" case, and distinguish an unknown revision from other failures.
4. src/ui.rs:154-166 Save happens only after a poll (every 10s) and on clean quit. Fine for v1, but a hard kill loses up to 10s of feed state, so commits fed in that window will be fed again on next start only if `last_seen` was not saved. Consider saving right after feeding (currently dirty is set, so this already happens on poll; only decay-only ticks are unsaved, which is harmless). No action required beyond awareness.
5. src/pet.rs:78 `sanitize` does not bound `last_update`, `total_fed` or `recent` ids; harmless but `recent[].summary` can be arbitrarily long and is rendered unwrapped. Truncate summaries on ingest in `parse_log` (e.g. 200 chars).
6. Tests: no test for `Mood` thresholds (Happy/Content/Hungry boundaries) or for `feed` ordering with `recent`. Add small unit tests in src/pet.rs.
