# Round 2 adversary (commit 47cfab8)

Reran `cargo test`: all suites pass (18 unit, integration suites 11/2/3/8/11). Working tree has only untracked review docs and an unrelated `.claude/agents/coder.md` edit.

## Reviewer claims

1. Branch-switch dedup limited to MAX_RECENT (20): CONFIRMED (src/lib.rs:32-36, src/pet.rs:24). Understated in one respect. The dedup only helps when the switch-back happens within 20 feeds. Worse, `feed()` and the dedup both ignore that a branch switch to a long-lived branch feeds up to MAX_NEW_COMMITS (1000) author commits in a single poll, which jumps total_fed and the stage (egg to adult). The "fix" is a heuristic, not the behavior the deferred item asked for. Still MINOR, since the spec only says "since the last seen commit".
2. Lowercased save key on all platforms: CONFIRMED (src/store.rs:27). MINOR is right. On Windows it is correct and desirable. Effect on Linux and macOS is a shared pet between case-differing paths, which is rare.
3. Load via `Value`, only `last_update` defaulted: CONFIRMED, and the reviewer's "check" is answered here. `last_update` is the only time-based field in `Pet` (src/pet.rs:48-62), so the other defaults are harmless. Two gaps the reviewer did not state. `"last_update": null` or a wrong type is not defaulted and falls to the corrupt path (fresh pet plus .bak). That is acceptable per spec. Because of `#[serde(default)]` plus the manual patch, `Default for Pet` (`Pet::new(0)`) is never what load relies on for time. A future second timestamp field would silently get 0 and massive decay. A comment on `Default` would help. MINOR/informational.
4. Test coverage gap for the `cat-file -e` path and `GitError::Command`: CONFIRMED. I found no test that forces either path. The vanished-baseline behavior (rewritten history re-baselines without feeding) is spec-adjacent ("only walk new commits") and is now unverified by tests.

The reviewer's "no BLOCKING" conclusion is correct.

## Missed findings

A. MINOR (unverified by execution, from reading src/git.rs:121-130). The new `cat-file -e` pre-check treats any non-zero exit as "baseline vanished" and silently re-baselines with no commits fed. Causes such as "dubious ownership" or a corrupt object store, or a transient failure, would produce the same exit status. A pet could silently skip real commits, which is the opposite of the error-surfacing the commit claims. Fix: inspect stderr, or use `git rev-parse --verify --quiet`, and only treat the specific missing-object case as vanished.

B. MINOR. Errors from `sync` (src/ui.rs:207) only set `message`, which is appended to the one-line key hint (src/ui.rs:128). A long git stderr (multi-line, wide) will be clipped or garbled on that single row, so the surfaced error may be unreadable. The error is also retried every poll with no backoff. It is not fatal, and `last_seen` is not advanced, so no commits are lost.

C. MINOR. On a sync error, `pet.tick(now)` has already run in `sync` and `dirty` is not set (src/ui.rs:207), so the decayed state is not saved until the next success or quit. The final save on quit covers it. Harmless.

D. MINOR/informational. `parse_log` truncation is a char count and does not strip control characters or ANSI escapes from the summary. A commit subject containing escape sequences is passed to ratatui. ratatui generally sanitizes cell contents, but this is unverified. The new tab/escape handling is not tested.

E. Terminal cleanup, checked. `ratatui::init` installs a panic hook, `restore` runs on every return path, and Ctrl-C is handled as a key event in raw mode (src/ui.rs:187). An external SIGINT or console-close event on Windows is not hooked. In raw mode this is normally delivered as a key, so acceptable.

F. Spec gaps, checked and none new. Non-git directory, empty repo, corrupt save with .bak, and clock-backwards handling (`tick` resets `last_update` and applies no decay) are all present.

## New BLOCKING findings

None.
