VERDICT: REVISE

# 003 Rust academy: round 1 verdict

Gatekeeper: Alakazam. Branch `feature/003-rust-academy` at `4facbf9`. Inputs: `round-1-review.md` (Jirachi), `round-1-adversary.md` (Gengar).

## Quality gates (C3), run by the gatekeeper
1. `cargo build`: pass.
2. `cargo clippy --all-targets -- -D warnings`: pass.
3. `cargo fmt --check`: pass.
4. `cargo test`: pass. Every suite is green: lib 48, academy_content 9, academy_grade 10, academy_progress 2, academy_project 4, academy_reference 1 (71 s), plus all 001/002 suites.
5. App starts: partly verified. `target/debug/gitagochi.exe --nope` runs and prints usage, exit 2. That path exits before the data dir is read. I did not open the full TUI: on Windows `dirs::data_dir()` cannot be redirected, and a real run writes `pet.json`/`progress.json` in the user's data dir (C4, orchestrator instruction). The reviewer did launch the TUI for 5 s without a crash. The plan's manual Verification still needs a real interactive run (see R3).

## Resolving reviewer vs adversary
- B1 (quiz passable by always answering 1): confirmed. Authored answer indices per module: m05 9/10 at 0, m06 8/10, m07-m10 10/10. `src/learn.rs:456-457` renders choices in authored order. LRN-18 is met in letter, but LRN-05/LRN-08 ("the assessment shall test the objectives") and the spec's Why are not met in substance on four modules. This is BLOCKING. How to fix it (shuffle or rebalance) is the owner's call under S2. The fix is required either way.
- B2 module 1: confirmed. `curriculum/hidden/m01.rs:11` needs whitespace-only lines treated as blank. Module 1 never teaches line iteration, `lines()`, `trim()`, or loops, which arrive in m02. BLOCKING (LRN-05).
- B2 module 4 (`splitn`): I side with the adversary. It is solvable with m03's taught slicing plus `split`/`next`, so MINOR. It is cheap to fix together with R2.
- M5 (CLAUDE.md): fixed by `4facbf9`, but the T034 box is still unticked. T035 is not done.

## Required items (BLOCKING)

### R1. Quiz answers must not be guessable by position (LRN-05, LRN-08, LRN-18)
- Where: `src/learn.rs:456-457` (choice rendering), and the quiz scoring path that compares against `Item::answer`. Content in `src/academy/content/m05.rs` .. `m10.rs`.
- Expected: no single key wins a quiz. Do one of the following:
  - (a) Shuffle each item's choices per attempt (reuse `quiz::order` with the attempt seed), and score against the shown permutation.
  - (b) Rebalance the authored `answer` positions in every module.
- Add a test so this cannot come back:
  - (a): a test that a retake permutes the choices and still scores correctly.
  - (b): a `tests/academy_content.rs` test that fails if any module has more than 4 of its answers at one index.
- After the fix, answering `1` (or any one key) to every item must score below 80% on every module.

### R2. Module 1's project check must use only what module 1 teaches (LRN-05, LRN-04)
- Where: `src/academy/content/m01.rs` ("Cargo projects" section, around the lesson body before the brief at ~line 140). The check is in `curriculum/hidden/m01.rs:10-16`.
- Expected: either teach the needed tools in module 1, or simplify the step:
  - Teach: add a short passage to a module 1 lesson section that covers `for line in log.lines()`, `line.trim().is_empty()` and a counter. Tag a quiz item to it if it becomes part of an objective.
  - Simplify: change the step and hidden test to something module 1 teaches.
- `tests/academy_reference.rs` and `tests/academy_content.rs` must stay green.
- Recommended in the same change (MINOR, not required): teach `splitn(4, '|')` in m04's "Pattern matching" section (B2 module 4), and add one sentence to m03 that a shared borrow ends at its last use (M1). The `lines()` passage also covers N1 (m03 `authors`).

### R3. Finish the task list (tasks.md, C3)
- Where: `docs/specs/003-rust-academy/tasks.md:125-126`.
- Expected:
  - Tick T034. Its work is done in `7348c19` and `4facbf9`.
  - For T035: run the C3 gates after R1/R2 and tick it. The manual Windows Verification in `plan.md` (steps 1-6) uses the real data dir, so an agent cannot run it under C4. The orchestrator must ask the owner to run those steps, or to accept the automated coverage as a stand-in. The box is ticked only after one of the two.

## Deferred (minor)
- m04 `parse_commit` needs `splitn`/`split_once`, which no lesson up to m04 teaches. It is solvable with taught slicing (`curriculum/hidden/m04.rs:20`; `src/academy/content/m04.rs`).
- M1 `src/academy/content/m03.rs:76-82`: the E0502 spot-the-bug relies on "a borrow ends at its last use", which the lesson does not state.
- M2 `src/academy/curriculum.rs:148-159` `checks_upto`: a deduplicated Fmt/Clippy keeps module 1's tags, so a module 10 clippy failure points to module 1's section. Merge the tags.
- M3 `src/academy/grade.rs:357-360` `probe` ignores `cancel` (60 s timeout per probe), so quitting during a probe can stall.
- M4 `src/learn.rs:151`: `p` during a quiz silently abandons it.
- M6 `src/academy/content/m09.rs:136,141`: the brief says "characters", but the lesson's example uses `.len()` (bytes).
- N1 `curriculum/hidden/m03.rs:29`: `authors` needs line iteration, which is first taught in m05. Covered if R2 adds a `lines()` passage.
- N2 `src/academy/grade.rs:377-388,424`: a project folder inside another repo (no own `.git`) exports the enclosing repo's HEAD. Return `NotARepo` when `repo.root()` is not the project folder. Not reachable via scaffold.
- N3 `src/academy/scaffold.rs:50-60`: if `git init` fails after the files are written, the folder is left non-empty and a retry is refused.
- New: the `--help`/usage text in `src/main.rs`/`src/lib.rs` still says "Keys: q quit, r refresh." and does not mention `l` (LRN-09 discoverability).

## Proposed known limitations
None yet. If the owner rules on S1-S3 as "not required", add the matching lines:
- "A user with no watched git repository cannot open the learn screen; the app exits with 'no git repositories found' before the academy project exists." (S1)
- "The project check trusts the output of the learner's own test binary; a project that prints fake libtest result lines can pass hidden tests it does not satisfy." (S3)

## Spec questions for the owner
- S1 (LRN-09, LRN-12): Must the academy be reachable by a new user with no watched repo? Today `run()` (`src/lib.rs`, after `repos::resolve`) exits with "no git repositories found", so a newcomer cannot open the learn screen to scaffold the project that would become their first repo.
- S2 (LRN-18): Should quiz choices be shuffled per attempt, or should the authored answer positions be balanced? Either one satisfies R1. Deciding this picks which option the coder implements.
- S3 (LRN-15): Must grading resist a learner who games it (for example, by printing fake `test NAME ... ok` lines)? The spec is silent and has no anti-cheat non-goal.
