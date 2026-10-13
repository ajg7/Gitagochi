# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Gitagochi is a terminal-based Tamagotchi-like game whose pet "feeds" on git commits (per README.md). It is written in Rust.

## Status

Round 1 implementation has landed (edition 2024). The requirements live in `docs/spec.md`; reviewers judge the code against it. Usage: `gitagochi [PATH ...] [--author EMAIL]` (paths may be repos or parent folders; else `repos.json` in the data dir); keys `q`/Esc/Ctrl-C quit, `r` refresh and re-scan. One shared pet is fed by all watched repos. A repo seen for the first time only baselines HEAD (existing history does not feed the pet); later commits by the author feed it.

## Commands

Toolchain: stable Rust via rustup (1.99.0 at time of writing; update with `rustup update stable`). Includes clippy and rustfmt.

```
cargo build                  # debug build
cargo run                    # run the app
cargo build --release
cargo test                   # all tests
cargo test <name>            # single test (substring match on test name)
cargo test --test <file>     # one integration test file in tests/
cargo clippy --all-targets -- -D warnings   # lint
cargo fmt                    # format (use `cargo fmt --check` in CI)
```

## Workflow: build and review loop

Substantial work runs through five subagents defined in `.claude/agents/`: `coder`, `reviewer`, `adversary`, `gatekeeper`, and `tester`. The main session orchestrates; it does not write the code itself.

Each round N (max 3):
1. `coder` implements (round 1) or fixes the items in `docs/review/round-(N-1)-verdict.md`, then commits.
2. `reviewer` reviews the diff and writes `docs/review/round-N-review.md`.
3. `adversary` reviews that review and the code, and writes `docs/review/round-N-adversary.md`.
4. `gatekeeper` runs the checks and writes `docs/review/round-N-verdict.md`, whose first line is `VERDICT: SATISFIED` or `VERDICT: REVISE`.

After a `VERDICT: SATISFIED`, run `tester` once. It writes tests from `docs/spec.md` (not from the implementation), edits only test code, and writes `docs/review/tester-report.md`. If it reports BLOCKING bugs, treat them as the required items of a new round (back to `coder`, then the usual reviewer/adversary/gatekeeper, then `tester` again) while rounds remain under the cap; otherwise escalate to the user. Tester work does not get its own extra rounds beyond the cap of 3.

Definition of satisfied: the app builds and runs, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test` pass, and the only remaining issues are minor and non-blocking.

If the verdict is REVISE, start the next round with only the verdict's required items passed to the coder. If round 3 still ends in REVISE (`ESCALATE: yes`), stop and give the user the open blocking items instead of looping. Pass state between agents through the files in `docs/review/`, not through chat, and don't share the coder's reasoning with the reviewers.

## Architecture

Crates: `ratatui` (re-exports `crossterm`), `serde`/`serde_json`, `dirs`; dev: `tempfile`. Git is read by shelling out to the `git` binary (no `git2`).

- `src/main.rs`: thin; parses args, prints errors, sets exit code.
- `src/lib.rs`: `sync()` (one repo), `sync_repo`, `sync_all` (decay once, feed from every repo), arg parsing, `run()` wiring.
- `src/pet.rs`: pure pet state and rules (hunger, health, stage, mood, clock-safe `tick`). No I/O. Also holds per-repo tracking: `repos: BTreeMap<canonical path, RepoTrack>` (baseline HEAD, fed-commit ids).
- `src/git.rs`: `Repo` (open, `from_root`, canonical `key()`, `name()`, head, default author, `commits_since` using `git log` on `last..HEAD`).
- `src/repos.rs`: `repos.json` config parsing and `resolve()` of paths into repos: a repo, a parent folder (scanned 3 levels deep, stops at repos, skips hidden dirs and node_modules/target/venv/.venv/__pycache__, no symlinks), or a folder inside a repo.
- `src/store.rs`: one shared save `pet.json` plus `repos.json` in the platform data dir; `migrate_legacy` adopts the newest old per-repo save (`<name>-<hash>.json`); corrupt files moved to `.bak`; atomic writes.
- `src/ui.rs`: ratatui rendering and event loop (poll 10s, re-scan repos every 60s or on `r`); terminal restored via `ratatui::restore` and the panic hook from `ratatui::init`.
- `tests/`: integration tests against real temporary git repos; `multi_repo.rs` covers multi-repo feeding, scanning and migration.
