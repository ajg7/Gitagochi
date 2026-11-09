# 003 Rust academy: tasks

Work top to bottom. Each task is small enough for one commit, leaves the quality gates green, and names the requirements it serves. The coder ticks a box in the same commit that finishes the task. `[P]` marks tasks that don't depend on the one before them.

There is one exception to "gates green". A task titled "Write failing tests" may leave only its own new tests red (they may not compile yet), and the very next task must turn them green. Clippy and fmt stay green throughout.

Design details (types, commands, the per-module API table) are in `plan.md`; follow it rather than re-deciding.

## Phase 1: Foundations
- [x] T001 Add curriculum types (`ObjId`, `Topic` + `Topic::ALL`, `Objective`, `Section`, `ItemKind`, `Item`, `CheckKind`, `Check`, `Module`), an empty `MODULES`, the lookups `objective`, `section` and `checks_upto` (deduplicates Build/Fmt/Clippy, keeps module order), and wire up `pub mod academy` — files: `src/lib.rs`, `src/academy/mod.rs`, `src/academy/curriculum.rs` — serves: LRN-01, LRN-02, LRN-07, LRN-16
- [x] T002 Write the curriculum alignment tests over `MODULES`; with no modules yet they pass vacuously and guard every later content task — files: `tests/academy_content.rs` — serves: LRN-02, LRN-05, LRN-06, LRN-07, LRN-08, LRN-17, LRN-18. Tests:
  - `lrn_02_a_every_module_complete`: objectives, sections, brief, quiz and checks are all non-empty.
  - `lrn_05_a_objectives_name_nonempty_sections`.
  - `lrn_06_a_items_taught_in_lessons`, plus `read_more` holds only `https://doc.rust-lang.org/book/` URLs.
  - `lrn_07_a_tags_known_and_not_later`: no empty tags; `ObjId.0` ≤ the module number, and the module's own objective ids equal `(n, 1..)`.
  - `lrn_08_a_every_objective_assessed`.
  - `lrn_18_each_quiz_has_both_kinds`: every quiz has at least one `MultipleChoice` and one `SpotTheBug` item, `answer < choices.len()`, and `choices.len() >= 2`.
  - `lrn_17_hidden_tests_match_checks`: every `Test(name)` has a `fn name(` in `hidden_tests` and the reverse; names are prefixed `mNN_` and unique; no `assert` in the hidden source.
- [x] T003 [P] Write failing tests for progress — files: `src/academy/progress.rs` (tests module), `tests/academy_progress.rs` — serves: LRN-11, LRN-14, LRN-22, LRN-23, LRN-24. Tests:
  - `lrn_11_open_rule`;
  - `lrn_14_a_both_parts_needed`;
  - `lrn_22_a_failed_retake_keeps_pass`;
  - `lrn_23_a_restart_restores_progress`: save, load, module 1 passed, module 2 open, project path remembered;
  - `lrn_24_a_corrupt_progress_backed_up`: invalid JSON leaves a `.bak` and module 1 open; also covers the missing-file case;
  - a sanitize test: unknown module keys dropped, `quiz_best` clamped to 100.
- [x] T004 Implement `Progress`/`ModuleProgress` (`passed`, `open`, `record_quiz`, `record_project_pass`, `badges`, `sanitize`). In `store.rs`, factor `write_atomic` out of `save` (same behavior) and add `progress_path`, `load_progress` and `save_progress` using `backup()` — files: `src/academy/progress.rs`, `src/store.rs` — serves: LRN-11, LRN-14, LRN-22, LRN-23, LRN-24
- [ ] T005 [P] Write failing tests for the quiz — files: `src/academy/quiz.rs` (tests module) — serves: LRN-18, LRN-20. Tests:
  - `lrn_18_a_eight_of_ten_passes_seven_fails`;
  - `lrn_20_a_retake_order_differs`: across 1000 seeds `order(n, Some(prev), seed) != prev` for n ≥ 2, and the result is always a permutation;
  - `score` reports the missed item indices.
- [ ] T006 Implement `score`/`Score` (integer 80% rule) and `order` (xorshift64 Fisher-Yates, rotated by one if equal to `prev`) — files: `src/academy/quiz.rs` — serves: LRN-18, LRN-19, LRN-20
- [ ] T007 [P] Write failing test `lrn_27_timeout_kills`: `run_capped` on `git hash-object --stdin` with stdin held open returns `Exit::TimedOut` within timeout + 1 s; a quick `git --version` returns `Exit::Code(0)` with stdout captured; output beyond `OUTPUT_CAP` is truncated — files: `src/academy/proc.rs` (tests module) — serves: LRN-27
- [ ] T008 Implement `proc::run_capped(cmd, timeout, cancel) -> io::Result<RunOut>`: stdin null unless the caller set it; two capped reader threads; 50 ms `try_wait` loop; kill then wait on timeout or cancel; detach readers after a kill — files: `src/academy/proc.rs` — serves: LRN-27
- [ ] T009 [P] Add the starter project assets and write failing scaffold tests — files: `curriculum/starter/{Cargo.toml.in,lib.rs,main.rs,gitignore}`, `tests/academy_project.rs` — serves: LRN-12, LRN-13. Tests:
  - `lrn_12_a_scaffold_is_cargo_git`: a new folder (path with a space) gets `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `.gitignore`, and `Repo::open` succeeds;
  - `lrn_13_a_non_empty_folder_refused_unchanged`: the directory listing and file contents are identical before and after; a path that is a file is also refused.
- [ ] T010 Implement `scaffold::create(dir)` and `git::init(dir)` (`git init -q` as an argument list). No initial commit — files: `src/academy/scaffold.rs`, `src/git.rs` — serves: LRN-12, LRN-13
- [ ] T011 [P] Write failing test `lrn_15_a_export_ignores_worktree`: commit a file, then modify it, add an untracked file and stage another change. `export_head` must write only the committed content, and `git status --porcelain` must be identical before and after — files: `tests/academy_grade.rs` — serves: LRN-15
- [ ] T012 Implement `git::export_head(root, dest, index_file) -> Result<String, GitError>` (returns the sha): `read-tree <sha>` and `--work-tree <dest> checkout-index -a -f`, both with `.env("GIT_INDEX_FILE", index_file)`. No commits gives an error the grader maps to `NoCommits` — files: `src/git.rs` — serves: LRN-15, LRN-26

## Phase 2: Module 1 and grading
- [ ] T013 Module 1 "Cargo and tooling" — files: `src/academy/content/m01.rs`, `curriculum/hidden/m01.rs`, `curriculum/solutions/step01/`, `src/academy/curriculum.rs` (`MODULES`) — serves: LRN-01, LRN-02, LRN-03, LRN-04, LRN-05, LRN-06, LRN-07, LRN-08, LRN-18. Contents:
  - objectives 1.1-1.3, a lesson section per objective, Rust Book `read_more`;
  - a brief (lib+bin crate `tracker`, the log format, `git log --format=%h|%ae|%ct|%s --output=log.txt`, commit to grade);
  - 10 quiz items (≥ 2 spot-the-bug);
  - checks Build[1.1], `m01_counts_non_empty_lines`[1.1], `m01_empty_log_is_zero`[1.1], Fmt[1.3], Clippy[1.3];
  - hidden tests using a local `check()` helper (no `assert`);
  - a step01 solution that is fmt- and clippy-clean.
- [ ] T014 Write failing grading tests. Each builds a temp repo from `curriculum/solutions/step01` (git run with `-c user.email=... -c user.name=... -c commit.gpgsign=false`) and uses a `GradeEnv` with temp work and target dirs and short timeouts — files: `tests/academy_grade.rs`, `src/academy/curriculum.rs` (tests module) — serves: LRN-15, LRN-16, LRN-17, LRN-25, LRN-26, LRN-27. Tests:
  - `lrn_15_a_grades_commit_not_worktree`: step01 committed, `lib.rs` broken in the working tree; passes.
  - `lrn_15_b_clippy_warning_names_lint`: step01 plus a committed `pub fn extra() -> u8 { return 1; }`; fails, and the clippy message contains `clippy::needless_return`.
  - `lrn_17_a_failure_report_hides_source`: `count_lines` committed as returning 0; the failing result has name `m01_counts_non_empty_lines`, a non-empty message and tags [1.1], and no trimmed hidden-source line of ≥ 15 chars appears in any message.
  - `lrn_25_a_missing_cargo_named`: cargo program `gitagochi-no-such-cargo`; `ToolMissing` whose message contains `cargo` and `rustup`; `MODULES[0]` sections and quiz still score through `quiz::score`.
  - `lrn_26_a_plain_folder_not_cargo`, plus a missing folder and a cargo folder with `git init` but no commits, each giving its own error variant and message.
  - `lrn_27_a_compile_error_shown`: a committed type error; Build fails with `error[E0308]` in its message, and the Test checks fail with "does not build".
  - `lrn_27_a_infinite_loop_times_out`: `count_lines` is `loop {}`, test timeout 5 s; the m01 test checks fail with "timed out".
  - `lrn_16_checks_upto_includes_earlier`: unit test with synthetic modules; module 3's checks hold modules 1-3's checks in order, with Build/Fmt/Clippy once.
- [ ] T015 Implement `grade.rs` per plan section 2:
  - `GradeEnv::new(data_dir)` holding the constants;
  - `GradeError` with `Display` messages (missing folder, not a cargo project, not a git repo, no commits, Cargo.toml not committed, tool missing + install hint, git/io);
  - preconditions, toolchain probe, export, Build/Fmt/Clippy via cargo JSON parsing (only lines starting with `{`);
  - per-module hidden-test compile (`message.message` only) and direct run of the test binary with the timeout;
  - libtest pretty-output parsing (an unseen name counts as failed);
  - message truncation and `strip_control`;
  - work dir cleanup.

  — files: `src/academy/grade.rs`, `src/git.rs` (`strip_control` becomes `pub(crate)`) — serves: LRN-15, LRN-16, LRN-17, LRN-25, LRN-26, LRN-27
- [ ] T016 Write failing test `lrn_27_a_spawn_does_not_block`: `spawn` returns at once, `rx.try_recv()` is `Empty` right after, and a result arrives later; setting `cancel` on a looping project ends the job within 2 s — files: `tests/academy_grade.rs` — serves: LRN-27
- [ ] T017 Implement `grade::spawn` → `Job { rx, cancel, handle }` on a `std::thread` — files: `src/academy/grade.rs` — serves: LRN-27
- [ ] T018 Write `lrn_04_a_reference_solutions`. For every module N in `MODULES` (steps in parallel via `std::thread::scope`, each with its own target dir): committed `stepNN` passes `grade(.., N)`, and committed `step(N-1)` (step 0 = the starter assets) fails at least one of module N's own checks. With only module 1 present it must pass now, and it guards every later content task — files: `tests/academy_reference.rs` — serves: LRN-04, LRN-16

## Phase 3: Learn screen and wiring
- [ ] T019 Write failing tests for the learn screen — files: `src/learn.rs` (tests module) — serves: LRN-01, LRN-09, LRN-10, LRN-11, LRN-17, LRN-19, LRN-20, LRN-22. Tests:
  - `lrn_09_a_l_opens_list_esc_returns`: `Nav::Pet` + `l` gives the list; Esc gives `Nav::Pet`.
  - `lrn_10_a_lesson_brief_quiz_check_reachable`: from a module menu, Lesson, Brief and Quiz are reachable, and Check gives `Action::StartCheck(1)`, or a prompt to press `p` when no project is set.
  - `lrn_11_a_module_2_locked`: Enter on module 2 with module 1 not passed stays on the list and shows a locked message.
  - `lrn_19_a_result_names_missed_objective_and_section`: `quiz_lines` holds the score and, for a missed item, its objective text and section title.
  - `lrn_20_a_retake_after_fail`: after a failed result, `r` starts the quiz again with a different order.
  - `lrn_17_a_check_lines_show_name_message_section`: `check_lines` on a failing `CheckResult` shows the name, the message, and "revisit: module 1, <section>".
  - `lrn_01_a_lists_all_modules_in_order`: TestBackend render of the list shows every `MODULES` title in order, with locked/open/passed markers.
  - `lrn_22_a_passed_module_reopens`: a passed module can be opened and its quiz retaken.
- [ ] T020 Implement `learn.rs`:
  - `Nav` (Pet, List, Menu, Reader for lesson/brief with scroll, Quiz, QuizResult, Check with Running/Done, Setup with a path input);
  - `Action`;
  - `handle_key`;
  - `draw` (ratatui `Paragraph` with wrap and scroll, `List`);
  - `quiz_lines` and `check_lines`.

  — files: `src/learn.rs`, `src/lib.rs` (`pub mod learn`) — serves: LRN-09, LRN-10, LRN-11, LRN-17, LRN-19, LRN-20, LRN-22
- [ ] T021 Write failing wiring tests — files: `src/ui.rs` (tests module), `tests/academy_project.rs` — serves: LRN-12, LRN-21. Tests:
  - `lrn_21_a_badge_shown_pet_unchanged`: TestBackend render of the pet screen with module 1 passed shows `[1]`, and the `Pet` before and after is equal.
  - `lrn_12_a_project_is_watched`: `resolve(&watched_paths(paths, &progress))` contains the scaffolded repo, including when the CLI or config paths do not list it.
  - `lrn_12_b_project_commit_feeds_pet`: scaffold, `sync_all` (baseline), commit as the author, `sync_all` → fed 1.
- [ ] T022 Wire up the app:
  - `lib::watched_paths`, and `run()` loads progress, builds `GradeEnv` and fills the new `Session` fields;
  - in `ui.rs`: the `l` key, `Nav` dispatch and drawing, a badges line in the pet panel, the `l: learn` hint;
  - carry out the actions: `SaveProgress` → `store::save_progress`; `StartCheck` → `spawn`, poll `try_recv` each frame, `record_project_pass` and save on pass; `Scaffold` → `create`, set `progress.project`, save, push to the live paths and force a rescan;
  - on quit, cancel and join any running job.

  — files: `src/lib.rs`, `src/ui.rs` — serves: LRN-09, LRN-12, LRN-21, LRN-23, LRN-27

## Phase 4: Curriculum content
Each module task adds the module per the plan's table and the same content list as T013:
- objectives, a lesson section per objective, Rust Book `read_more`, a brief that names the exact public API to add;
- 10 quiz items, at least 2 spot-the-bug, tags per objective;
- checks tagged with the objectives they exercise;
- `curriculum/hidden/mNN.rs` (a `check()` helper, no `assert`, library API only);
- `curriculum/solutions/stepNN/`: step N-1 plus the feature, fmt- and clippy-clean, keeping every earlier public path;
- the module registered in `MODULES`.

`tests/academy_content.rs` and `tests/academy_reference.rs` must stay green after each one. Every content task serves LRN-01, LRN-02, LRN-04, LRN-05, LRN-06, LRN-07, LRN-08, LRN-16, LRN-18, plus the listed LRN-03 topic.
- [ ] T023 Module 2 "Types, variables and control flow" (`day_of`, `longest_streak`) — files: `src/academy/content/m02.rs`, `curriculum/hidden/m02.rs`, `curriculum/solutions/step02/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (types, variables, control flow)
- [ ] T024 Module 3 "Ownership and borrowing" (`field`, `authors`, `normalize`) — files: `src/academy/content/m03.rs`, `curriculum/hidden/m03.rs`, `curriculum/solutions/step03/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (ownership and borrowing)
- [ ] T025 Write `lrn_16_a_module_3_names_broken_module_1_check`: step03 with `pub fn count_lines(` renamed to `count_lines_old(` and `pub fn count_lines(_log: &str) -> usize { 0 }` appended, committed; `grade(.., 3)` fails, and a failing result has module 1 and name `m01_counts_non_empty_lines`. It passes on arrival because T015 implemented inheritance; it needed module 3's content to exist — files: `tests/academy_grade.rs` — serves: LRN-16
- [ ] T026 Module 4 "Structs, enums and pattern matching" (`Commit`, `Kind`, `parse_commit`) — files: `src/academy/content/m04.rs`, `curriculum/hidden/m04.rs`, `curriculum/solutions/step04/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (structs, enums and pattern matching)
- [ ] T027 Module 5 "Error handling with Result and ?" (`ParseError`, `parse_line`, `parse_log`, main without unwrap) — files: `src/academy/content/m05.rs`, `curriculum/hidden/m05.rs`, `curriculum/solutions/step05/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (error handling)
- [ ] T028 Module 6 "Collections and iterators" (`commits_per_author`, `top_authors`, `streak`) — files: `src/academy/content/m06.rs`, `curriculum/hidden/m06.rs`, `curriculum/solutions/step06/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (collections and iterators)
- [ ] T029 Module 7 "Traits and generics" (`Stat`, `TotalCommits`, `LongestStreak`, `report`, `Display for Commit`, `most_common`) — files: `src/academy/content/m07.rs`, `curriculum/hidden/m07.rs`, `curriculum/solutions/step07/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (traits and generics)
- [ ] T030 Module 8 "Modules, visibility and testing" (`pub mod parse`, `pub mod stats`, root `pub use`) — files: `src/academy/content/m08.rs`, `curriculum/hidden/m08.rs`, `curriculum/solutions/step08/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (modules, visibility and testing)
- [ ] T031 Module 9 "Lifetimes" (`Fields<'a>`, `split_fields`, `longest_summary`) — files: `src/academy/content/m09.rs`, `curriculum/hidden/m09.rs`, `curriculum/solutions/step09/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (lifetimes)
- [ ] T032 Write failing test `lrn_03_a_every_topic_has_objective` (every `Topic::ALL` variant is the topic of a module with ≥ 1 objective), plus a test that `MODULES.len() == 10`; red until T033 — files: `tests/academy_content.rs` — serves: LRN-03, LRN-01
- [ ] T033 Module 10 "Idiomatic Rust" (`pet` module with `Pet`/`Mood`, `Display` + `Error` for `ParseError`, `FromStr for Commit`; checks include Fmt[10.3] and Clippy[10.3]) — files: `src/academy/content/m10.rs`, `curriculum/hidden/m10.rs`, `curriculum/solutions/step10/`, `src/academy/curriculum.rs` — serves: as above, LRN-03 (idiomatic best practices), LRN-15

## Phase 5: Polish
- [ ] T034 Update the README and CLAUDE.md: the `l` key, the learn-screen keys, project setup, `progress.json` and `<data>/academy/`, the new modules in Architecture, and the requirement that the Rust toolchain be on PATH to grade — files: `README.md`, `CLAUDE.md` — serves: LRN-09, LRN-12, LRN-23, LRN-25
- [ ] T035 Run all quality gates (C3) and the manual check in `plan.md` Verification on Windows — serves: C3, LRN-09, LRN-27
