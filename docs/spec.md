# Gitagochi spec (DRAFT, needs owner review before round 1)

A terminal Tamagotchi-like pet that feeds on git commits. Items marked **(assumption)** are defaults I chose; change them before the build loop starts.

## Core behavior

- Run `gitagochi` inside a git repo (or pass a path) and a single pet lives in the terminal UI.
- **Feeding:** each new commit by the configured author (default: `git config user.email`) since the last seen commit feeds the pet. **(assumption)**
- **Hunger:** rises with real time; decays the pet's health if it stays hungry too long. Feeding lowers hunger.
- **Mood/stage:** derived from hunger, health and total commits fed. Stages: egg, baby, adult. **(assumption)**
- **Death:** none in v1; a starving pet becomes "sleepy" and recovers when fed. **(assumption)**
- **Commit quality:** all commits feed equally in v1; no message or diff analysis. **(assumption)**

## UI

- Full-screen TUI: pet drawn in ASCII, status bars (hunger, health), recent fed commits list, key hints.
- Keys: `q` quit, `r` refresh commits now. Also polls for new commits periodically. **(assumption)**
- Terminal must be restored on quit, panic and Ctrl-C.

## Persistence

- Pet state saved as JSON under the platform data dir (e.g. `%APPDATA%` on Windows), one file per repo path. **(assumption)**
- Missing or corrupt save file starts a fresh pet and does not crash; the corrupt file is kept as a `.bak`.
- Time-based decay is computed from saved timestamps, so closing the app does not pause the pet, and clock going backwards is handled without panics.

## Multi-repo (v1.1)

One pet can be fed by several repos at once. This supersedes "one file per repo path" and the single-repo wording above, while keeping single-repo use working.

- **Selecting repos:** `gitagochi PATH [PATH ...]` accepts several paths. With no paths, repos come from a config file `repos.json` under the platform data dir (same dir as saves): `{"repos": ["C:\\...\\GameDev", "..."], "author": "you@example.com"}`. Paths given on the command line override the config's `repos`; `--author` overrides the config's `author`.
- **Parent folders:** a path that is not itself a git repo is scanned for repos below it, up to 3 levels deep (e.g. `Tech & Coding/Coding_Project/Gitagochi` is found from `Tech & Coding`). The scan does not descend into a directory that is a repo (so nested repos and submodules are not double-counted), and it skips hidden directories and `node_modules`, `target`, `venv`/`.venv` and `__pycache__`. It is re-evaluated periodically (not on every 10s poll if that is slow; at least every minute) so new repos are picked up. Non-repo directories are skipped silently. Symlinks and junctions are not followed, to avoid cycles.
- **Errors:** if a configured path is missing or not a repo, skip it and show a one-line warning in the UI; other repos still feed. It is an error (clear message, exit code 1) only if no repo at all can be resolved.
- **One shared pet:** all selected repos feed the same pet and share one save file, independent of which repos are selected. Selecting a different set of repos on a later run keeps the same pet.
- **Per-repo tracking:** the save stores last-seen commit, and fed-commit dedup, per repo (keyed by canonical repo path). A repo seen for the first time only baselines its HEAD (existing history does not feed), exactly as in single-repo mode. Rewritten history or a vanished baseline re-baselines only that repo.
- **Feeding:** each repo's new commits by the configured author feed the pet equally. The caps for large histories apply per repo.
- **UI:** the recent-commits list shows which repo each commit came from, and the header shows how many repos are being watched.
- **Migration:** an existing single-repo save (the old per-repo-path file) is adopted as the shared pet's state, with that repo's tracking carried over, so existing pets are not reset. If several old saves exist, use the most recently modified one.
- **Tests:** unit tests for config parsing and path resolution (explicit repos, parent folder scan, missing paths); integration tests with two real temp repos feeding one pet, per-repo baselines, a repo added or removed between runs, and old-save migration.

## Non-goals for v1

Multiple pets, networking, sound, config UI, commit content analysis.

## Architecture and quality bar

- As in CLAUDE.md: thin `main.rs`, logic in `lib.rs`, pure pet logic with no terminal or git I/O.
- Unit tests for pet logic (feeding, decay, stage transitions, clock edge cases) and for save/load.
- Works on Windows first; must not rely on Unix-only APIs.
- Handles: non-git directory (clear error), repo with no commits, very large history (only walk new commits).
