# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Gitagochi is a terminal-based Tamagotchi-like game whose pet "feeds" on git commits (per README.md). It is written in Rust.

## Status

Features 001 (core pet) and 002 (multi-repo) are implemented (edition 2024); 003 (Rust academy) is in progress. Development is spec-driven: rules for every feature are in `docs/constitution.md`, each feature's requirements are in `docs/specs/NNN-<feature>/spec.md` (IDs like `CORE-04`, `MR-07`), and the lifecycle is in `docs/specs/README.md`. Reviewers judge code against the spec. Usage: `gitagochi [PATH ...] [--author EMAIL]` (paths may be repos or parent folders; else `repos.json` in the data dir); keys `q`/Esc/Ctrl-C quit, `r` refresh and re-scan, `l` opens the learn screen (Rust academy; Esc back, `p` project setup, Ctrl-C always quits). The academy project folder (from `progress.json`) is always watched too. Grading needs cargo, rustfmt and clippy on PATH; lessons and quizzes do not. One shared pet is fed by all watched repos. A repo seen for the first time only baselines HEAD (existing history does not feed the pet); later commits by the author feed it.

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

Substantial work runs through six subagents defined in `.claude/agents/`: `planner`, `coder`, `reviewer`, `adversary`, `gatekeeper`, and `tester`. The main session orchestrates; it does not write the code itself. Below, `F` is the feature folder `docs/specs/NNN-<feature>/`, and every agent gets `F` and `docs/constitution.md`.

Before the loop:
1. Spec: use the `specify` skill with the user until `F/spec.md` says `Status: Approved`. Never start the planner on a Draft spec.
2. `planner` writes `F/plan.md` and `F/tasks.md` and ends the plan's Analysis with `Result: READY` or `BLOCKED`. If BLOCKED, take its questions to the user and update the spec first. Set the spec to `Status: In progress`.

Each round N (max 3):
1. `coder` works through `F/tasks.md` (round 1) or fixes the items in `F/review/round-(N-1)-verdict.md`, then commits.
2. `reviewer` reviews the diff and writes `F/review/round-N-review.md`.
3. `adversary` reviews that review and the code, and writes `F/review/round-N-adversary.md`.
4. `gatekeeper` runs the checks and writes `F/review/round-N-verdict.md`, whose first line is `VERDICT: SATISFIED` or `VERDICT: REVISE`.

After a `VERDICT: SATISFIED`, run `tester` once. It writes tests from the spec's acceptance criteria (not from the implementation), edits only test code, and writes `F/review/tester-report.md`. If it reports BLOCKING bugs, treat them as the required items of a new round (back to `coder`, then the usual reviewer/adversary/gatekeeper, then `tester` again) while rounds remain under the cap; otherwise escalate to the user. Tester work does not get its own extra rounds beyond the cap of 3.

Definition of satisfied: the app builds and runs, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test` pass, and the only remaining issues are minor and non-blocking.

If the verdict is REVISE, start the next round with only the verdict's required items passed to the coder. If round 3 still ends in REVISE (`ESCALATE: yes`), stop and give the user the open blocking items instead of looping. Pass state between agents through the files in `F/`, not through chat, and don't share the coder's reasoning with the reviewers.

Closing a feature: with the user's approval, add the verdict's "Proposed known limitations" to `F/spec.md`, then set `Status: Implemented`. If a later fix changes agreed behavior, update the spec in the same change. `docs/review/` holds the archived reviews of 001 and 002 from before this layout.

## Architecture

Crates: `ratatui` (re-exports `crossterm`), `serde`/`serde_json`, `dirs`; dev: `tempfile`. Git is read by shelling out to the `git` binary (no `git2`).

- `src/main.rs`: thin; parses args, prints errors, sets exit code.
- `src/lib.rs`: `sync()` (one repo), `sync_repo`, `sync_all` (decay once, feed from every repo), arg parsing, `watched_paths` (adds the academy project), `run()` wiring.
- `src/pet.rs`: pure pet state and rules (hunger, health, stage, mood, clock-safe `tick`). No I/O. Also holds per-repo tracking: `repos: BTreeMap<canonical path, RepoTrack>` (baseline HEAD, fed-commit ids).
- `src/git.rs`: `Repo` (open, `from_root`, canonical `key()`, `name()`, head, default author, `commits_since` using `git log` on `last..HEAD`, `init`, `export_head` (HEAD tree into a folder through a private `GIT_INDEX_FILE`, leaving the repo untouched)). `strip_control` is `pub(crate)`.
- `src/repos.rs`: `repos.json` config parsing and `resolve()` of paths into repos: a repo, a parent folder (scanned 3 levels deep, stops at repos, skips hidden dirs and node_modules/target/venv/.venv/__pycache__, no symlinks), or a folder inside a repo.
- `src/store.rs`: one shared save `pet.json` plus `repos.json` in the platform data dir (overridden by a non-empty `GITAGOCHI_DATA_DIR` env var, see `default_data_dir`); `migrate_legacy` adopts the newest old per-repo save (`<name>-<hash>.json`); corrupt files moved to `.bak`; atomic writes (`write_atomic`); `progress.json` via `load_progress`/`save_progress`.
- `src/academy/`: the Rust course. `curriculum.rs` (types, `MODULES`, `checks_upto`), `content/mNN.rs` (module data), `progress.rs` (pure pass/lock rules), `quiz.rs` (80% scoring, retake order), `proc.rs` (capped, timed, cancellable child runs), `grade.rs` (grade the last commit: preconditions, toolchain probe, export, build/fmt/clippy, hidden tests run as their own binary with a timeout; `spawn` runs it on a worker thread; data in `<data>/academy/`), `scaffold.rs` (starter project + `git init`).
- `curriculum/`: assets outside the crate: `starter/` (embedded), `hidden/mNN.rs` (hidden tests, embedded; no `assert`, use a `check()` helper), `solutions/stepNN/` (reference solutions, tests only).
- `src/learn.rs`: learn screen state (`Learn`, `Nav`), pure `handle_key` returning an `Action`, drawing, `quiz_lines`/`check_lines`.
- `src/ui.rs`: ratatui rendering and event loop (poll 10s, re-scan repos every 60s or on `r`); terminal restored via `ratatui::restore` and the panic hook from `ratatui::init`. Dispatches keys through `learn::Learn`, draws badges, runs one grading `Job` at a time (polled each frame, cancelled and joined on quit).
- `tests/`: integration tests against real temporary git repos; `multi_repo.rs` covers multi-repo feeding, scanning and migration. `academy_*.rs` cover the academy: `academy_content` (curriculum alignment rules), `academy_reference` (each step solution passes its module, the previous step fails; runs real cargo, about a minute), `academy_grade`, `academy_project`, `academy_progress`.
