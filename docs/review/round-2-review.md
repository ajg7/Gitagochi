# Round 2 review (commit 47cfab8)

Checks run by reviewer: cargo build OK; clippy -D warnings clean; fmt --check clean; cargo test all pass (18 unit + integration).

No BLOCKING findings.

## MINOR

1. src/lib.rs:32-36 - branch-switch dedup only checks `pet.recent`, which is capped at MAX_RECENT (src/pet.rs:23). After more than MAX_RECENT other feeds, switching back to an old branch can still re-feed. Mitigation, not a complete fix. Fix: ancestry check (`git merge-base --is-ancestor`) or a bounded set of fed ids persisted in Pet. Document the limitation otherwise.
2. src/store.rs:27 - save key is still lowercased on all platforms (deferred item 6 second half), so two repos differing only by case on Linux share a pet. Fix: lowercase only under `cfg!(windows)`.
3. src/store.rs:50-61 - load goes through serde_json::Value and patches `last_update` manually; a corrupt-but-valid-JSON field still falls back to a fresh pet (acceptable). Only `last_update` gets a "now" default; with `#[serde(default)]` any other missing field takes `Pet::new(0)` values (e.g. born time 0 if such a field exists). Fine if none are time-based; check.
4. tests/git_switch.rs - covers branch switching and non-git dir. Deferred item 5 asked also for empty repo, author filter, and invalid `since`; tests/git_edge.rs and git_sync.rs appear to cover some of these, but no test exercises the new `cat-file -e` vanished-baseline path or the new `GitError::Command` surfacing in commits_since.
