# Gitagochi constitution

Rules that apply to every feature. A spec or plan may not override them; changing one is its own change, approved by the owner, with a line in the amendment log at the bottom. Every agent reads this file before it starts.

## C1. Spec first
- No behavior is built until its spec in `docs/specs/NNN-<feature>/spec.md` has `Status: Approved`.
- The spec says *what* and *why*. It does not name crates, functions or files; those belong in `plan.md`.
- Every requirement has an ID and at least one acceptance criterion that a test can check.
- When code and spec disagree, the spec wins until the owner changes it. A fix that changes agreed behavior updates the spec in the same commit.
- Known limitations that are accepted, not fixed, are written into the spec's "Known limitations" section.

## C2. Architecture
- Thin `src/main.rs` (args, errors, exit code). Logic lives in `src/lib.rs` and its modules.
- `src/pet.rs` is pure: no terminal, git or filesystem I/O.
- Git is read by running the `git` binary with an argument list, never a shell string. No `git2`.
- Minimal dependencies; each new crate is justified in the plan.

## C3. Quality gates (all must pass before a round is SATISFIED)
1. `cargo build`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo fmt --check`
4. `cargo test`
5. The app starts.

## C4. Safety
- No `unwrap`/`expect` on paths a user can reach; return `Result`.
- The terminal is always restored on quit, panic and Ctrl-C.
- Save files are written atomically, never trusted, and a corrupt file is moved to `.bak` instead of crashing.
- No secrets or personal data in logs or error messages beyond what the user configured.
- Tests never touch the real user data dir or the real repo's git config.

## C5. Platform and scale
- Windows first; no Unix-only APIs.
- Only new commits are read; memory stays bounded for very large histories and many repos.
- Behavior is correct across restarts and when the clock goes backwards.

## C6. Code style
- Comments only when absolutely necessary, at most 2 lines.
- Tests are written from the spec's acceptance criteria and name the requirement ID they cover (e.g. `// MR-07` or a test name containing `mr_07`).

## Amendment log
- 2026-11-08: Constitution created from CLAUDE.md and the agent rules.
