# Round 3 review (commit e31b79e)

Checks run by me: cargo build OK; clippy -D warnings clean; fmt --check clean; cargo test all pass (20 unit, integration suites 11/4/3/8/11/9, 0 ignored). The previously ignored branch-switch test in tests/round2.rs now runs and passes.

No BLOCKING findings. The tester's BLOCKING item (branch-switch refeed) is fixed: sync() dedups against a persisted `fed_ids` set (cap 10,000), backfilled from `recent` for old saves in `normalize`. Deferred items 1, 4 (partly), 5, 6 and 7 are addressed.

## MINOR
1. src/pet.rs:61-65 / persistence: `fed_ids` is saved in full (up to 10,000 x ~40 bytes, about 400 KB) on every dirty save. Acceptable, but a lower cap (e.g. 2,000) or skipping unchanged saves would keep the file small.
2. src/pet.rs `has_fed` (new pub fn) is not used by `sync`, which builds its own HashSet. Use it or remove it. `normalize` does `fed_ids.contains` in a loop (O(20 x n)), fine at this size.
3. src/git.rs strip_control: a bare ESC not followed by `[` drops only the ESC, so OSC sequences (`ESC ] 0;title BEL`) leave `]0;title` visible. Harmless (no control effect), cosmetic only.
4. src/git.rs:123-130: with `rev-parse --verify --quiet`, any non-empty stderr (e.g. a git warning on Windows such as safe.directory/CRLF notices) on a vanished baseline becomes a persistent GitError instead of a re-baseline. Consider treating only exit code 1 with empty stderr as "vanished" and other codes as errors (current logic is close; just be aware of warnings).
5. Deferred item 2 (src/store.rs lowercase key only under cfg!(windows)) and 3 (Default comment) remain unaddressed; no impact on Windows.
6. Test gap: no integration test for the GitError::Command path in `commits_since` (non-empty stderr from rev-parse); only unit tests for strip_control and short_error were added. tests/git_switch.rs covers the dedup case.
