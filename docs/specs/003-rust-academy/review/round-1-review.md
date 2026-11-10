# 003 Rust academy: round 1 review

Reviewer: Jirachi. Diff: `git diff 49e317a..HEAD` (22 commits, through 7348c19).

## Quality gates (run by the reviewer)
- `cargo build`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo fmt --check`: pass.
- `cargo test`: pass. All suites green, including `academy_reference` (90 s) and `academy_grade` (19 s).
- App starts: `target/debug/gitagochi.exe --author ... .` ran for 5 s without exiting or crashing, then hit the timeout. The learn screen was not driven interactively, so the manual Verification steps in plan.md (T035) were not re-run.

## Requirement status
| ID | Status | Evidence / note |
|----|--------|-----------------|
| LRN-01, 02, 03 | met | 10 modules in `MODULES`, `lrn_01_ten_modules_shipped`, `lrn_02_a`, `lrn_03_a` |
| LRN-04 | partly met | LRN-04.a passes, but step 1 and step 4 need techniques that no lesson up to that point teaches (B2) |
| LRN-05 | not met | Module 1's project checks depend on untaught content (B2). The tag-level test passes, but the content does not satisfy it |
| LRN-06, 07, 08 | met at tag level | Tags are well formed and every objective is assessed. Spot checks of the quiz items found each tagged item answerable from its section (one MINOR gap, m3) |
| LRN-09, 10, 11 | met | `learn.rs` tests; the event loop keeps ticking while the learn screen is open |
| LRN-12, 13 | met | `academy_project.rs` |
| LRN-14 | met | `progress::passed` |
| LRN-15, 16, 17 | met | `academy_grade.rs`; `failure_text` drops the `thread ... academy_m` panic location line |
| LRN-18 | not met in substance | The quiz can be passed without reading anything (B1) |
| LRN-19..24 | met | |
| LRN-25, 26, 27 | met | |

## BLOCKING

### B1. The quiz can be passed by pressing `1` every time (LRN-18, LRN-05/08, the spec's "a pass means you learned it")
Choices are shown in authored order (`src/learn.rs:455-457`), and the authored correct answers are heavily skewed toward index 0:

```
m05: 0 1 0 0 0 0 0 0 0 0      (9/10 are choice 1)
m06: 1 0 0 0 0 1 0 0 0 0      (8/10)
m07..m10: all 0               (10/10)
```

Pressing `1` ten times scores 100% on modules 7-10 and at least 80% on modules 5-6. The correct choice is also often the longest and most specific one. Item order is shuffled (LRN-20), but the choice order is not, so the shuffle does not help. As it stands, a quiz pass shows nothing about the objectives.

Fix: shuffle each item's choices per attempt with the existing `quiz::order`/seed, remap `answer`, and score against the shown permutation. Alternatively, rebalance the authored answer positions. Either way, add a content test so this cannot come back. For example, in `tests/academy_content.rs`, fail if any module has more than 4 of its 10 answers at one index. If choices are shuffled instead, test that a rendered retake permutes them.

### B2. Project checks assess skills no lesson has taught (LRN-05, LRN-04)
I compared each reference solution's method calls with the lesson and quiz text of modules 1..=N. The two gaps below decide pass or fail.

- **Module 1** (`src/academy/content/m01.rs`; `curriculum/hidden/m01.rs:10-17`). `m01_counts_non_empty_lines` requires counting lines and treating whitespace-only lines (`"   "`) as blank. Module 1 teaches only cargo, tests, fmt and clippy, with nothing on iterating text. `str::lines` first appears in module 5's lesson, and `str::trim` is not taught in any module. The check is tagged 1.1 ("Build, run and test a crate..."), but passing it depends on string APIs from outside the curriculum. Fix: add a short passage on the needed tools to the "Cargo projects" section (`for line in log.lines()`, `line.trim().is_empty()`, a counter), or simplify the step to something module 1 teaches.
- **Module 4** (`curriculum/hidden/m04.rs:20`, `"fix: a | b"`). `parse_commit` must keep later `|` characters in the summary. Lessons 1-4 teach only `split`/`nth`. The reference uses `splitn(4, '|')`, which is never taught, and `split_once` first appears in module 9. Fix: teach `line.splitn(4, '|')` in m04's "Pattern matching" section, next to the existing `let (Some(a), Some(b)) = (parts.next(), parts.next()) else` example.

`tests/academy_content.rs` checks only that tags name a non-empty section, so it cannot catch this class of gap. That is expected, but it means the content has to be reviewed by hand, as done here.

## MINOR

- M1 `src/academy/content/m03.rs` (spot-the-bug E0502, "Print first before calling v.push(9)"), LRN-05. The "References and borrowing" section shows the error but never says that a shared borrow ends at its last use, and the correct fix depends on that. Add one sentence to the section.
- M2 `src/academy/curriculum.rs:153-158` (`checks_upto`), LRN-17. When Fmt/Clippy are deduplicated, module 10's tags `[10.3]` are dropped and module 1's `[1.3]` are kept. A clippy failure while grading module 10 therefore says "revisit: module 1, Formatting and linting" instead of "Clippy-clean idioms". Suggest merging the tags of the deduplicated checks rather than keeping only the first.
- M3 `src/academy/grade.rs` `probe` uses its own `never` flag with a 60 s timeout. If the user quits during the probe, `cancel_and_join` (`src/ui.rs`) can block quit for up to 60 s. Pass `cancel` through to `probe`.
- M4 `src/learn.rs:151`: `p` in a quiz silently abandons it for Setup. Probably harmless, but Esc is the documented way out.
- M5 T034 is half done: README is updated, CLAUDE.md is not (no learn keys, no `src/academy`/`src/learn.rs` in Architecture, no `progress.json`). T034 and T035 are still unticked in `tasks.md`.
- M6 `src/academy/content/m09.rs` brief says "by characters", but the lesson's `longer` example uses `.len()` (bytes). The hidden tests are ASCII-only, so either passes. Make the brief say "by length" or add a note on `chars().count()`.

## Unrequested behavior
None of substance. `Pet::starve` (m10) and `longer` (m09) go beyond the plan's API table but serve those modules' objectives.

Not checked: the plan.md Verification steps were not driven interactively (Windows TestBackend tests cover key handling and rendering); the parts of `proc.rs` past line 120 were only skimmed.
