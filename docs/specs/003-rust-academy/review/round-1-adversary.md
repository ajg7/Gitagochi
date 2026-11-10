# 003 Rust academy: round 1 adversarial review

Adversary: Gengar. Inputs: `round-1-review.md`, `git diff 49e317a..HEAD`. Note: HEAD moved to `4facbf9` ("T034: document the Rust academy in CLAUDE.md") after the review was written; that commit touches only `CLAUDE.md`.

Gates re-run by me: `cargo fmt --check` pass, `cargo clippy --all-targets -- -D warnings` pass, `cargo test --test academy_content` pass (9/9). I did not re-run the full suite or the app: on Windows `dirs::data_dir()` cannot be redirected, so starting the app would touch the real data dir (C4).

## Reviewer claims

| Claim | Verdict | Evidence |
|---|---|---|
| Gates pass | CONFIRMED (fmt, clippy, academy_content re-run) | see above |
| B1 quiz passable by pressing `1` | CONFIRMED | `grep -oE 'answer: [0-9]+' src/academy/content/m*.rs`: m05 `0 1 0 0 0 0 0 0 0 0`, m06 `1 0 0 0 0 1 0 0 0 0`, m07..m10 all `0`. `src/learn.rs:455-457` renders choices in authored order. Severity note below. |
| B1 sub-claim "the correct choice is often the longest" | UNVERIFIED | I did not measure it. |
| B2 module 1 needs untaught `lines`/`trim` | CONFIRMED | `curriculum/hidden/m01.rs:11` has a `"   "` line; no lesson in m01 mentions `lines`, `trim`, `split` or any string iteration. A scan of methods used by `curriculum/solutions/stepNN` against the text of modules 1..=N lists `.lines .trim` as untaught for step 1, and `.trim` as untaught in every module through 10. There is no in-curriculum route to "whitespace-only line is blank" at module 1. BLOCKING is right. |
| B2 module 4 needs untaught `splitn` | OVERSTATED | The gap is real, but the step is solvable with taught material: m03's "String slices" teaches byte-range slicing (`&owned[0..2]`, `src/academy/content/m03.rs:99`), and m03/m04 teach `split`/`next`. A learner can take three fields with `next()` and slice `&line[h.len() + a.len() + t.len() + 3..]`. Awkward, not impossible, so MINOR. Teaching `splitn` (or `split_once`) in m04 is still the cheap fix. |
| M1 m03 E0502 fix needs "borrow ends at last use" | CONFIRMED | `m03.rs:76-82` states the rule and shows the error but never says a shared borrow ends at its last use, which the correct choice at `m03.rs:219-221` depends on. MINOR. |
| M2 `checks_upto` keeps module 1's tags for Fmt/Clippy | CONFIRMED | `src/academy/curriculum.rs:148-159`: the first Fmt/Clippy wins, later tags (`[10.3]`) are dropped. Still names a real section, so LRN-17 is met in letter. MINOR. |
| M3 `probe` ignores cancel | CONFIRMED | `src/academy/grade.rs:357-360` uses a local `never` flag with `PROBE_TIMEOUT` 60 s; up to three probes run, so quit could block up to 180 s in the worst case (normally under a second). MINOR. |
| M4 `p` in a quiz abandons it | CONFIRMED | `src/learn.rs:151` handles `p` before the `Nav::Quiz` arm. MINOR. |
| M5 CLAUDE.md not updated | WRONG (now) | Fixed by `4facbf9` after the review. The rest stands: T034 and T035 are still `[ ]` in `tasks.md:125-126`. |
| M6 m09 brief says characters, lesson uses `.len()` | CONFIRMED | Brief `m09.rs:136,141` says "characters", lesson example `m09.rs:38` uses `.len()`, reference uses `chars().count()` (`step09/src/stats.rs:149`). Tests are ASCII-only, so either passes. MINOR. |
| LRN-09..27 "met" | CONFIRMED by reading for LRN-09, 11, 14, 15, 17, 20, 22, 23, 24, 26 | Pet sync runs before the draw regardless of `Nav` (LRN-09); `RUST_BACKTRACE=0` and the `academy_m` location filter keep hidden source out of failures (LRN-17); `record_quiz` keeps the max (LRN-22); `load_progress` backs up invalid or non-object JSON (LRN-24); integer pass rule `5c >= 4t` agrees with `percent >= 80` for any item count (LRN-18). |

### Severity of B1
LRN-18 as written ("multiple-choice and spot-the-bug items, pass at 80%") is met literally; the defect is against the spec's Why ("a pass means the learner can do what the module set out to teach") and LRN-05/08's intent. The project part still gates each module (LRN-14), so pressing `1` alone cannot pass a module. I still agree with BLOCKING: on modules 7-10 the quiz measures nothing, and the fix is small. If the gatekeeper reads BLOCKING strictly as "requirement text not met", treat B1 as a spec gap for the owner (see S2) rather than downgrading it silently.

## New findings

### MINOR
- N1 (LRN-05, LRN-04) Module 3's `authors(log)` (`curriculum/hidden/m03.rs:29`) also needs line iteration over a multi-line log with a blank line; `lines()` is first taught in m05 (`m05.rs:79`). Weaker than B2 because `split('\n')` can be inferred from m03's `split('|')` and `is_empty` is taught. Fix with the same passage added for B2 module 1.
- N2 (LRN-26, LRN-15) `grade` checks `project/Cargo.toml` but exports `repo.root()` (`src/academy/grade.rs:377-388`, `424`). If the project folder is a subfolder of some other repo (no `.git` of its own), the enclosing repo's HEAD is exported: the result is a misleading `NotCommitted`, or, if that repo's root has its own `Cargo.toml`, the wrong crate is graded. Not reachable through the scaffold (it runs `git init` in the folder), only via a hand-edited `progress.json` or a deleted `.git`. Suggest returning `NotARepo` when `repo.root()` is not the project folder (compare canonical paths). Found by reading, not reproduced.
- N3 (LRN-13) `scaffold::create` (`src/academy/scaffold.rs:50-60`) writes the starter files before `git init`. If `git init` fails, the folder is left non-empty, and a retry is refused as `NotEmpty`. Rare (git is required by the app anyway). Either init first, or say in the error that the folder can be removed and retried.

### Checked and fine (no finding)
- Shared `CARGO_TARGET_DIR` across `work-<pid>` dirs does not accumulate artifacts: I built the same `tracker` crate from two different dirs into one target dir and got a single `libtracker-ac7c91add29ba7cd.rlib` (scratchpad repro).
- CRLF: global `core.autocrlf=true` on this machine and the test repos do not override it, so the grade tests already run on CRLF exports.
- Ctrl-C is handled before `learn.handle_key` (`src/ui.rs`), quit cancels and joins the job, and the terminal restore path is unchanged.
- Tests do not touch the real data dir or git config (`-c user.email=...` throughout; no `dirs::` in `tests/academy_*`).

## Spec gaps (owner decides)
- S1 (LRN-12) A new user with no watched repo cannot reach the learn screen: `run()` still exits with "no git repositories found" when `resolve` returns nothing (`src/lib.rs` after `watched_paths`), and the academy project only exists after setup. The spec does not say whether the course must be reachable without any repo. Read from code; not run (C4).
- S2 (LRN-18) The spec does not say whether choice order should be randomized or answer positions balanced. Deciding this settles B1's classification.
- S3 (LRN-15) The spec is silent on whether a learner's own output can affect grading (for example, writing fake `test NAME ... ok` lines directly to stdout). Not a bug under the current spec; the non-goals say nothing about anti-cheat.

## Summary
Reviewer's two BLOCKING items hold: B1 confirmed; B2 confirmed for module 1, overstated for module 4 (solvable with taught slicing, so MINOR). All MINORs confirmed except M5, which `4facbf9` fixed (T034/T035 boxes still unticked). New: three MINORs (m03 line iteration, grading a project inside another repo exports the wrong root, scaffold leaves a non-empty folder if `git init` fails) and three spec gaps (no-repo users can't open the academy, choice-order policy, self-reported test output).
