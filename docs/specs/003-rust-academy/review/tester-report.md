# Tester report: 003 Rust academy (spec v1.3, after round-4 SATISFIED and T036)

Result: no BLOCKING bugs found. Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (see the end of this file for the final run).

New tests: `tests/academy_spec.rs` (new), `tests/academy_grade.rs` (added `lrn_28_c_earlier_pass_does_not_carry_over`). No non-test code changed.

## Coverage

| Criterion | Test | Result |
|---|---|---|
| LRN-01.a | academy_spec `lrn_01_a_all_modules_listed_in_order_offline`; academy_content `lrn_01_ten_modules_shipped` | pass (curriculum is compiled in; no network use) |
| LRN-02.a | academy_content `lrn_02_a_every_module_complete` | pass |
| LRN-03.a | academy_content `lrn_03_a_every_topic_has_objective` | pass |
| LRN-04.a | academy_reference (each step passes, previous step fails; real cargo) | pass |
| LRN-05.a | academy_content `lrn_05_a_objectives_name_nonempty_sections` | pass |
| LRN-06.a | academy_content `lrn_06_a_items_taught_in_lessons` | pass |
| LRN-07.a | academy_content `lrn_07_a_tags_known_and_not_later` | pass |
| LRN-08.a | academy_content `lrn_08_a_every_objective_assessed` | pass |
| LRN-09.a | academy_spec `lrn_09_a_l_opens_list_and_esc_returns` | pass (state level; "pet keeps updating" is in `ui::run` and not testable without a terminal) |
| LRN-10.a | academy_spec `lrn_10_a_all_four_views_reachable` | pass |
| LRN-11.a | academy_spec `lrn_11_a_module_2_locked_until_module_1_passed` | pass |
| LRN-12.a | academy_project `lrn_12_a_scaffold_is_cargo_git`, `lrn_12_a_project_is_watched` | pass |
| LRN-12.b | academy_project `lrn_12_b_project_commit_feeds_pet` | pass |
| LRN-13.a | academy_project `lrn_13_a_non_empty_folder_refused_unchanged` | pass |
| LRN-14.a | academy_spec `lrn_14_a_both_parts_required` | pass |
| LRN-15.a | academy_grade `lrn_15_a_grades_commit_not_worktree`, `lrn_15_a_export_ignores_worktree` | pass |
| LRN-15.b | academy_grade `lrn_15_b_clippy_warning_names_lint` | pass |
| LRN-16.a | academy_grade `lrn_16_a_module_3_names_broken_module_1_check` | pass |
| LRN-17.a | academy_grade `lrn_17_a_failure_report_hides_source` | pass |
| LRN-18.a | academy_spec `lrn_18_a_pass_mark_on_real_quizzes` (every shipped quiz: 80% passes, one fewer fails) | pass |
| LRN-19.a | academy_spec `lrn_19_a_missed_item_names_objective_and_section` | pass |
| LRN-20.a | academy_spec `lrn_20_a_retake_after_fail_reorders_items` | pass |
| LRN-20.b | academy_spec `lrn_20_b_fixed_position_strategies_never_certain` (every module, every fixed position, 60 seeds: some attempt fails; correct position varies) | pass |
| LRN-21.a | academy_spec `lrn_21_a_badge_visible_and_pet_unchanged` | pass |
| LRN-22.a | academy_spec `lrn_22_a_failed_retake_keeps_pass` | pass |
| LRN-23.a | academy_progress `lrn_23_a_restart_restores_progress` | pass |
| LRN-24.a | academy_progress `lrn_24_a_corrupt_progress_backed_up` | pass |
| LRN-25.a | academy_grade `lrn_25_a_missing_cargo_named` | pass |
| LRN-26.a | academy_grade `lrn_26_a_plain_folder_not_cargo` (also missing folder, no commits) | pass |
| LRN-27.a | academy_grade `lrn_27_a_compile_error_shown`, `lrn_27_a_infinite_loop_times_out`, `lrn_27_a_spawn_does_not_block` | pass |
| LRN-28.a | academy_grade `lrn_28_a_fake_test_output_does_not_pass` | pass |
| LRN-28.b | academy_grade `lrn_28_b_early_exit_zero_does_not_pass`, `lrn_28_c_earlier_pass_does_not_carry_over` (stale pass from a previous run) | pass |
| LRN-28 hardening gap | academy_spec `lrn_28_hidden_tests_are_instrumentable` (each hidden test appears exactly once as `fn name() {`) | pass |
| LRN-29.a | academy_e2e `lrn_29_a_no_paths_no_repo_opens_with_notice` (real binary, temp cwd and GITAGOCHI_DATA_DIR: stays alive 2s, screen output contains "no repos watched yet") | pass; pressing `l` is manual (no tty to send keys), covered at unit level |
| LRN-29.b | academy_e2e `lrn_29_b_explicit_plain_folder_exits_1`; unit test in `src/lib.rs` | pass |
| LRN-30.a | academy_e2e `lrn_30_a_files_land_under_data_dir_only` (pet.json under the data dir; platform data dir file list and mtimes unchanged; cwd untouched) | pass; only the files written in a 2s run are observed (progress.json, academy/ appear only on learn actions, manual) |

Standing cases (already covered by earlier features' tests, rechecked by name): non-git dir (`git_edge::non_git_directory_gives_clear_error`), no commits (`empty_repo_syncs_without_error_and_feeds_nothing`), very large history (`very_large_history_only_new_commits_are_walked`), corrupt save (`persistence::corrupt_variants_start_fresh_and_keep_bak`), clock backwards (`clock_going_backwards_does_not_panic_or_decay`), decay across restarts (`decay_continues_across_restart`), only the configured author feeds (`only_configured_author_feeds_and_case_is_ignored`). Academy edge cases: corrupt/missing progress, uncommitted edits, non-empty folder, later step breaking earlier feature, mis-tagged items are all in the table above. Windows paths with spaces: grade fixtures use `my project` and `data dir`.

## Untestable without a seam (report, not added)
- Resolved by LRN-30 (T036): LRN-29.a/b and LRN-30.a now run the real binary in `tests/academy_e2e.rs`. Remaining manual: press `l` on the no-repo start and see the learn screen (crossterm reads the console/tty, not piped stdin).
- LRN-09.a "the pet keeps updating while the learn screen is open": lives in the `ui::run` event loop; no seam.
- LRN-28 "non-zero exit with a marker present" (adversary M4): no deterministic black-box way to get a marker written and then a non-zero exit from learner code, because the marker path and nonce are not known to learner code except via the known limitation. Not tested.
- Manual TUI check on Windows (plan.md Verification) not performed by the tester.

## Bugs found
None blocking. All tests pass against the shipped code.

## Spec ambiguities
- LRN-20.b: "no such strategy reaches the pass mark with certainty" is tested probabilistically (60 seeds per position per module). Quizzes with few choices (2 or 3) give a fixed-position strategy a high per-attempt pass chance only if the quiz is very short; consider stating a minimum quiz size.
- LRN-29.a: none now; the seam exists.
- LRN-28: whether a learner reading the grader's files is in scope (verdict G1). The proposed known limitation in the round-4 verdict covers it.

Final run (after adding academy_e2e): fmt clean, clippy clean, cargo test all suites pass.
