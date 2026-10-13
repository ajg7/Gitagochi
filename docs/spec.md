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

## Non-goals for v1

Multiple pets, networking, sound, config UI, commit content analysis.

## Architecture and quality bar

- As in CLAUDE.md: thin `main.rs`, logic in `lib.rs`, pure pet logic with no terminal or git I/O.
- Unit tests for pet logic (feeding, decay, stage transitions, clock edge cases) and for save/load.
- Works on Windows first; must not rely on Unix-only APIs.
- Handles: non-git directory (clear error), repo with no commits, very large history (only walk new commits).
