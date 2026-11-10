# 003 Rust academy: plan

Spec: `spec.md` version 1.0
Author: planner

## Context
Gitagochi users commit code; this feature adds a built-in, offline Rust course whose project lives in a watched repo, so the learner's commits feed the pet through the existing path (CORE-01, MR-11, unchanged). It builds on: `git.rs` (argument-list git calls, `Repo`), `repos::resolve` (watched repos), `store.rs` (atomic writes, corrupt file to `.bak`), `ui.rs` (one event loop that polls every 250 ms and syncs every 10 s), and `serde_json` (already a dependency, reused here to parse cargo's JSON messages).

## Constitution check
- C1 Spec first: spec is `Status: Approved` v1.0, no `[NEEDS CLARIFICATION]`. Every LRN requirement maps to a task and an acceptance test (see Test plan and Analysis). Interpretations are listed under Analysis, not built silently.
- C2 Architecture: `main.rs` unchanged. `pet.rs` is not touched; badges come from a separate `Progress` value, so passing cannot change hunger, health or feeding (LRN-21). All git calls (`git init`, `read-tree`, `checkout-index`) go in `git.rs` through `Command` with argument lists. Cargo is also run with argument lists, never a shell. No new crates.
- C3 Quality gates: every task leaves build, clippy, fmt green. A "write failing test" task leaves only its own new tests red, and the next task turns them green. The academy tests run real `cargo`, `rustfmt` and `clippy`, which the gates already need.
- C4 Safety: no `unwrap`/`expect` on user paths. Grading errors are an enum turned into messages. Child processes get `stdin(Stdio::null())`, so they cannot read the raw-mode terminal. Grading runs on a worker thread that never touches the terminal, and `ratatui::restore` and the panic hook stay as they are. `progress.json` is written atomically with the `store.rs` helper, sanitized on load, and moved to `.bak` if corrupt. Output from the learner's program is passed through the existing `strip_control` before it is drawn. Messages show only the project path the learner chose. Tests pass in their own data, work and target dirs and the cargo program name. They never read the real data dir and never set env vars, since `set_var` is unsafe in edition 2024.
- C5 Platform and scale: the design is Windows first. The export uses only git plumbing (no `tar`, no symlinks). The timeout uses `try_wait` polling plus `kill`, which is portable std with no Unix APIs. Paths with spaces go in as separate `Command` args. Memory is bounded: captured output is capped per process and messages are truncated. Grading reads only the HEAD tree. The project's commits are read by the existing incremental sync. Progress has no time logic, so clock changes don't affect it. It is saved after every change, so it survives restarts.
- C6 Style: hidden tests and reference solutions are curriculum assets, not app code. App tests name their requirement IDs (`lrn_NN_x`).

## Approach

### 1. Curriculum as static Rust data plus included code assets
- Module text goes in `src/academy/content/mNN.rs` as `pub const MODULE: Module = Module { ... }`. Lesson bodies and briefs are raw string literals (`r#"..."#`). Being static data, nothing is parsed at runtime and no runtime error is possible (LRN-01: offline, ordered by position in `MODULES`).
- Code assets live outside `src/` so that our crate does not compile, format or lint them:
  - `curriculum/starter/` holds the step-0 project: `Cargo.toml.in`, `lib.rs`, `main.rs`, `gitignore`. The `.in` and no-dot names stop cargo from treating the folder as a nested package. These files are embedded with `include_str!`.
  - `curriculum/hidden/mNN.rs` holds each module's hidden tests, embedded with `include_str!`.
  - `curriculum/solutions/stepNN/` holds each step's reference solution (`Cargo.toml`, `src/lib.rs`, `src/main.rs`). Only tests use them; they are not embedded.
- Alignment rules are enforced by `tests/academy_content.rs`, which iterates over `MODULES`. Any mis-tagged, untaught or unassessed item fails `cargo test` (LRN-05..08, LRN-02.a, LRN-03.a). The same file checks that every `CheckKind::Test(name)` matches a `fn name(` in that module's hidden source and the reverse, that names are prefixed `mNN_` and unique, and that hidden sources contain no `assert`. Hidden tests use a local `check()` helper that panics with an authored message only, so a failure cannot echo test source (LRN-17).
- LRN-04.a is enforced by `tests/academy_reference.rs`. For each step N in `1..=MODULES.len()` (in parallel with `std::thread::scope`, each step with its own temp target dir), it commits step N into a temp repo and requires a full grade up to module N to pass. It then commits step N-1 (step 0 is the starter) and requires at least one of module N's own checks to fail.

### 2. Grading the last commit (LRN-15), with hidden tests (LRN-17), timeouts (LRN-27), off the UI thread
Done in `academy::grade::grade(project, modules, upto, env, cancel)`:
1. Preconditions, in order (LRN-26):
   - folder missing gives `FolderMissing`;
   - no `Cargo.toml` in the folder gives `NotCargo`;
   - `Repo::open` fails gives `NotARepo`;
   - `head()` is `None` gives `NoCommits`.
2. Toolchain (LRN-25): run `cargo --version`, then `cargo fmt --version` and `cargo clippy --version` if the checks include Fmt or Clippy. `NotFound` from spawn, or a non-zero exit, gives `ToolMissing { tool, hint }`. Hints are `https://rustup.rs`, `rustup component add rustfmt` and `rustup component add clippy`. Lessons and quizzes never call these tools.
3. Export HEAD (LRN-15) with `git::export_head(root, dest, index_file)`:
   - `GIT_INDEX_FILE=<work>/index git -C <root> read-tree <sha>`
   - `GIT_INDEX_FILE=<work>/index git -C <root> --work-tree <dest> checkout-index -a -f`

   A private index file means the learner's index, working tree and refs are never touched, and uncommitted or untracked files are never seen. I ran this locally with git 2.45.1 on Windows, in a path with a space: the export held the committed content, and `git status` in the source repo was identical before and after. If the export has no `Cargo.toml`, the result is `NotCommitted`.
4. Checks: the checks of modules `1..=upto`, in module order (LRN-16), with Build, Fmt and Clippy deduplicated. Each check runs in `<dest>` with `CARGO_TARGET_DIR=<academy>/target` (reused between runs for speed) and `CARGO_TERM_COLOR=never`:
   - Build: `cargo build --message-format=json`. On failure, the message is the `rendered` text of the error diagnostics (learner code, shown in full, up to the cap). If Build fails, every Test check fails with "project does not build; fix the build check first" (LRN-27).
   - Fmt: `cargo fmt --check`. On failure, the message is "run `cargo fmt`" plus the first diff lines.
   - Clippy: `cargo clippy --all-targets --message-format=json -- -D warnings`. On failure, the message names each lint by `message.code.code` (for example `clippy::needless_return`) followed by the rendered diagnostic (LRN-15.b).
   - Fmt and Clippy run before the hidden tests are injected, so they never see hidden code.
   - Tests: for each module m ≤ upto, write `curriculum/hidden/mMM.rs` to `<dest>/tests/academy_mMM.rs`, overwriting any learner file with that name in the export copy only. Then:
     - Compile with `cargo test --no-run --message-format=json --test academy_mMM` (build timeout). Take the `compiler-artifact` whose `profile.test` is true and whose `target.kind` contains `"test"`; its `executable` is the test binary.
     - If the compile fails, each of m's Test checks fails with only the diagnostics' `message.message` (for example "cannot find function `count_lines` in crate `tracker`"), never `rendered`, which would show hidden source lines.
     - Run the test binary directly with `--test-threads=1 --color never` (test timeout). Parse libtest's pretty output: `test NAME ... ok|FAILED`, and the failure text between `---- NAME stdout ----` and the next `----` or `failures:` line, with the `note: run with RUST_BACKTRACE` line dropped.
     - Running the binary ourselves instead of `cargo test` means the timeout kills the process that is actually looping. On Windows, `Child::kill` is `TerminateProcess`, which only ends the one process, so killing `cargo` would leave the test binary running and still locking its `.exe`.
     - On timeout, every check not yet reported `ok` fails with "timed out after Ns (infinite loop?)".
     - Compiling and running per module lets a later module's compile error not hide an earlier module's results, so LRN-16.a names the module 1 check.
5. Result: `Report { results: Vec<CheckResult> }`. `passed()` is true only if every result passed. `CheckResult` carries the module, name, passed flag, message and tags. The UI resolves the tags to objective text and section title (LRN-17).
6. Process helper `academy::proc::run_capped(cmd, timeout, cancel)`:
   - `stdin` null, `stdout` and `stderr` piped;
   - two reader threads, each keeping at most `OUTPUT_CAP` bytes and draining the rest;
   - a 50 ms `try_wait` loop that checks the deadline and the `cancel: &AtomicBool`, and calls `kill()` then `wait()` when either fires;
   - after a kill, the reader threads are detached instead of joined (a surviving grandchild may hold the pipe).
7. Non-blocking: `academy::grade::spawn(project, upto, env) -> Job { rx: mpsc::Receiver<Result<Report, GradeError>>, cancel: Arc<AtomicBool>, handle }`. The UI calls `try_recv` every frame. On quit it sets `cancel` and joins. Only one job runs at a time.
8. Work dir: `<academy>/work-<pid>/`. It is removed before and after each run (best effort), so parallel app instances don't collide. Constants: `BUILD_TIMEOUT` 300 s, `TEST_TIMEOUT` 30 s, `OUTPUT_CAP` 256 KiB, `MESSAGE_MAX_LINES` 40. They are defined once, at the top of `grade.rs`, as `GradeEnv::new(data_dir)` defaults. Tests build a `GradeEnv` with short timeouts.

### 3. Project setup (LRN-12, LRN-13)
- `academy::scaffold::create(dir)`:
  - refuses if `dir` exists and is not an empty directory, or is a file, without writing anything (`NotEmpty`);
  - otherwise creates the folder, writes the starter files (`Cargo.toml`, `src/lib.rs`, `src/main.rs`, `.gitignore` with `/target`), and calls `git::init(dir)` (`git init -q`).
- No initial commit is made, so it works without a git identity. The first commit, made by the learner, feeds the pet: an empty repo baselines with `last_seen = None`, and `commits_since(None)` then returns every commit (existing behavior).
- Writing the starter files directly instead of running `cargo new` means setup works without a toolchain (LRN-25).
- Watching the project: `progress.project` is appended to the watched paths every run (`lib::watched_paths`) and is pushed into the live session's paths with a forced rescan right after setup. It is not written into `repos.json`, because:
  - writing `repos.json` would turn a user's implicit "current directory" default (MR-02) into "only the project";
  - it would not help a run with CLI paths (MR-03);
  - it would need write support for a file that is currently read-only to us.

  `resolve` already deduplicates paths. See Analysis, interpretation 1.
- The folder is chosen in a one-line path prompt on the learn screen (`p`), prefilled with `dirs::home_dir()/gitagochi-tracker`.

### 4. Progress (LRN-23, LRN-24, LRN-11, LRN-14, LRN-22)
`academy::progress::Progress` is pure, with serde. `store::load_progress` and `store::save_progress` reuse `backup()` and a new shared `write_atomic()`, factored out of `save()` with the same behavior.

### 5. Learn screen (LRN-09, LRN-10, LRN-19, LRN-20)
- `src/learn.rs` holds the screen state, a pure key handler and the drawing code. `ui.rs` keeps the one event loop: it still ticks and syncs the pet every frame and poll (LRN-09: the pet keeps updating), and only the drawing and key dispatch depend on `Nav`.
- `learn::handle_key(&mut Nav, KeyCode, &Progress, &mut Rng) -> Action`, where `Action` is `None | Quit | SaveProgress | StartCheck(module) | Scaffold(PathBuf)`. The event loop carries out the side-effect actions.
- Keys:
  - Pet screen: `l` opens the learn screen.
  - Learn screens: Esc goes back one level; on the module list it returns to the pet screen.
  - Up/Down and Enter select.
  - In a quiz, `1`-`4` or `a`-`d` answer.
  - In a result view, `r` retakes or reruns.
  - `p` opens project setup.
  - `q` quits, except while typing a path. Ctrl-C always quits.
- The pet screen's hint gains `l: learn`, and the pet panel shows `Badges: [1] [2] ...` for passed modules (LRN-21).

## Data model and formats

```rust
// src/academy/curriculum.rs
pub struct ObjId(pub u8, pub u8);            // (module number, objective number), 1-based
pub enum Topic { Tooling, Basics, Ownership, Types, Errors, Collections, Traits, Modules, Lifetimes, Idioms }
pub struct Objective { pub id: ObjId, pub text: &'static str, pub section: &'static str }
pub struct Section  { pub title: &'static str, pub body: &'static str }
pub enum ItemKind { MultipleChoice, SpotTheBug }
pub struct Item  { pub kind: ItemKind, pub prompt: &'static str, pub choices: &'static [&'static str], pub answer: usize, pub tags: &'static [ObjId] }
pub enum CheckKind { Build, Fmt, Clippy, Test(&'static str) }
pub struct Check { pub kind: CheckKind, pub tags: &'static [ObjId] }
pub struct Module { pub title: &'static str, pub topic: Topic, pub objectives: &'static [Objective],
    pub sections: &'static [Section], pub read_more: &'static [&'static str], pub brief: &'static str,
    pub quiz: &'static [Item], pub checks: &'static [Check], pub hidden_tests: &'static str }
pub static MODULES: &[Module] = &[m01::MODULE, ...];   // module number = index + 1
```
`Topic::ALL` lists every variant and is used by the LRN-03.a test. A unit test matches on `Topic` exhaustively, so a new variant cannot be left out of `ALL`.

`progress.json` in the platform data dir (`store::progress_path(data_dir)`):
```json
{ "modules": { "1": { "quiz_best": 90, "project_passed": true } }, "project": "C:\\Users\\me\\gitagochi-tracker" }
```
- `Progress { modules: BTreeMap<u8, ModuleProgress>, project: Option<PathBuf> }`, with `#[serde(default)]` everywhere.
- `ModuleProgress { quiz_best: u8 /*percent*/, project_passed: bool }`.
- Derived rules:
  - `passed(m) = quiz_best >= 80 && project_passed` (LRN-14);
  - `open(m) = (1..m).all(passed)` (LRN-11);
  - `record_quiz(m, pct)` keeps the maximum, and `record_project_pass(m)` sets true; neither ever lowers a value, so a pass is never lost (LRN-22);
  - `badges()` lists the passed modules.
- `sanitize()`:
  - drops keys outside `1..=MODULES.len()`;
  - clamps `quiz_best` to 100;
  - turns an empty `project` path into `None`.
- A missing file gives `Progress::default()`, which is module 1 open (LRN-24). Invalid JSON, or JSON that is not an object, goes to `.bak` and then the default.
- There is no older format to migrate.

Quiz, `academy::quiz` (pure):
- `score(items, answers) -> Score { correct, total, percent, passed, missed: Vec<usize> }`, where `passed = correct * 5 >= total * 4` (integer math, LRN-18).
- `order(n, prev: Option<&[usize]>, seed: u64) -> Vec<usize>`: a Fisher-Yates shuffle using an xorshift64 seeded from the clock's nanoseconds. If the result equals `prev` and `n >= 2`, it is rotated by one, so a retake's order always differs (LRN-20). The last order is kept per module in memory.

Driving project contract. The crate is named `tracker`, with a lib and a bin. Hidden tests call only the library API. The log has one commit per line, `hash|author|unix_seconds|summary`, produced with `git log --format=%h|%ae|%ct|%s --output=log.txt`; `--output` avoids PowerShell's UTF-16 `>` redirection. Each step's API only adds to the last one, which keeps earlier hidden tests valid (LRN-16).

| # | Module (Topic) | Objectives (section) | Step adds (public API) | Hidden tests (`mNN_`) |
|---|---|---|---|---|
| 1 | Cargo and tooling (Tooling) | 1.1 build/run/test with cargo, lib+bin layout; 1.2 unit tests with `#[test]` and `assert_eq!`; 1.3 fmt and clippy clean | `count_lines(&str) -> usize` (non-empty lines); main prints it for `log.txt` | `counts_non_empty_lines`, `empty_log_is_zero`; checks also Build[1.1], Fmt[1.3], Clippy[1.3] |
| 2 | Types, variables, control flow (Basics) | 2.1 integer types, `let`/`mut`/shadowing; 2.2 `if`/`loop`/`while`/`for`; 2.3 functions and expressions | `day_of(ts: i64) -> i64`; `longest_streak(days: &[i64]) -> u32` (sorted input, repeats allowed) | `day_of_rounds_down`, `streak_counts_consecutive_days`, `streak_ignores_repeats`, `streak_empty_is_zero` |
| 3 | Ownership and borrowing (Ownership) | 3.1 moves, `Copy` vs `String`, `clone`; 3.2 `&`/`&mut` and the borrow rules; 3.3 `&str` slices | `field(line: &str, i: usize) -> &str` (`""` if missing); `authors(log: &str) -> Vec<String>` (unique, first-seen order); `normalize(names: &mut Vec<String>)` (lowercase in place) | `field_returns_slice`, `field_missing_is_empty`, `authors_unique_in_order`, `normalize_lowercases_in_place` |
| 4 | Structs, enums, pattern matching (Types) | 4.1 structs and `impl` methods; 4.2 enums and `Option`; 4.3 `match`/`if let` | `Commit { pub hash, pub author: String, pub time: i64, pub summary: String }` (Debug, Clone, PartialEq); `Kind { Feature, Fix, Docs, Other }`; `Commit::kind(&self)` from summary prefix `feat`/`fix`/`docs`; `parse_commit(&str) -> Option<Commit>` | `parse_commit_fields`, `parse_commit_rejects_short_line`, `kind_from_prefix` |
| 5 | Error handling (Errors) | 5.1 `Result` for recoverable errors; 5.2 `?` propagation; 5.3 custom error enums, no panics in library code | `ParseError { MissingField(usize), BadTime(usize) }` (1-based line, Debug, PartialEq); `parse_line(&str, usize) -> Result<Commit, ParseError>`; `parse_log(&str) -> Result<Vec<Commit>, ParseError>` (skips blank lines); main prints the error and exits non-zero | `parse_log_ok`, `reports_missing_field_line`, `reports_bad_time_line` |
| 6 | Collections and iterators (Collections) | 6.1 `Vec`, `HashMap`/`BTreeMap`; 6.2 iterator adapters and `collect`; 6.3 closures | `commits_per_author(&[Commit]) -> BTreeMap<String, usize>`; `top_authors(&[Commit], n) -> Vec<(String, usize)>` (count desc, then name asc); `streak(&[Commit]) -> u32` | `counts_per_author`, `top_authors_sorted`, `streak_from_commits` |
| 7 | Traits and generics (Traits) | 7.1 define and implement traits, `dyn`; 7.2 generics with bounds; 7.3 std traits (`Display`, derives) | `trait Stat { fn name(&self) -> String; fn value(&self, &[Commit]) -> String }`; `TotalCommits`, `LongestStreak`; `report(&[&dyn Stat], &[Commit]) -> Vec<String>` (`"name: value"`); `Display for Commit` (`"hash author: summary"`); `most_common<T: Ord + Clone>(&[T]) -> Option<T>` | `display_commit`, `stats_report`, `most_common_generic` |
| 8 | Modules, visibility, testing (Modules) | 8.1 `mod` and files; 8.2 `pub`, `pub use` re-exports; 8.3 unit and integration tests | `pub mod parse` (Commit, Kind, ParseError, parse_*), `pub mod stats` (the rest); root `pub use` keeps old paths | `parse_module_path`, `stats_module_path` |
| 9 | Lifetimes (Lifetimes) | 9.1 lifetime annotations on functions; 9.2 structs holding references; 9.3 elision rules | `Fields<'a> { pub hash, pub author, pub time, pub summary: &'a str }`; `split_fields<'a>(&'a str) -> Option<Fields<'a>>`; `longest_summary<'a>(&'a [Commit]) -> Option<&'a str>` | `split_fields_borrows`, `longest_summary`, `longest_summary_empty_none` |
| 10 | Idiomatic Rust (Idioms) | 10.1 std trait impls (`Default`, `Display`, `FromStr`, `Error`); 10.2 encapsulation and illegal states unrepresentable; 10.3 clippy-clean idioms (borrowed params, iterators, saturating math) | `pub mod pet { Pet (private hunger: u8), Default (hunger 50), feed(&mut self, n: usize) saturating, hunger(), mood() -> Mood { Happy, Hungry, Sleeping }, Display for Mood }`; `Display` + `std::error::Error` for ParseError; `FromStr for Commit` | `pet_default_and_feed_saturates`, `mood_display`, `commit_from_str`, `parse_error_is_std_error`; checks also Fmt[10.3], Clippy[10.3] |

- Every module has 10 quiz items, at least 2 of them spot-the-bug.
- Objectives that hidden tests cannot observe (1.2, 8.3, 9.3) are assessed by quiz items (LRN-08).
- A test's tags are the objectives its assertions exercise.
- `read_more` holds `https://doc.rust-lang.org/book/...` URLs only.

## Module changes
| File | Change | Public items | Requirements |
|------|--------|--------------|--------------|
| `src/lib.rs` | `pub mod academy; pub mod learn;`; `run()` loads progress, builds `GradeEnv`, uses `watched_paths`; `Session` gains progress fields | `watched_paths(paths, &Progress) -> Vec<PathBuf>` | LRN-12, LRN-23 |
| `src/academy/mod.rs` | new; declares submodules | | |
| `src/academy/curriculum.rs` | new; types above, `MODULES`, `Topic::ALL`, lookups `objective(ObjId)`, `section(m, title)`, `checks_upto(modules, n)` | as listed | LRN-01..08, LRN-16 |
| `src/academy/content/mNN.rs` (10) | new; module data | `MODULE` | LRN-01..08, LRN-18 |
| `src/academy/progress.rs` | new; pure progress rules | `Progress`, `ModuleProgress` | LRN-11, 14, 21, 22, 23 |
| `src/academy/quiz.rs` | new; pure scoring and ordering | `score`, `Score`, `order` | LRN-18, 19, 20 |
| `src/academy/proc.rs` | new; capped, timed, cancellable child run | `run_capped`, `RunOut`, `Exit` | LRN-27 |
| `src/academy/grade.rs` | new; preconditions, toolchain, export, checks, parsing, worker | `grade`, `spawn`, `Job`, `GradeEnv`, `Report`, `CheckResult`, `GradeError` | LRN-15, 16, 17, 25, 26, 27 |
| `src/academy/scaffold.rs` | new; starter project | `create`, `ScaffoldError` | LRN-12, 13 |
| `src/git.rs` | add `init(dir)`, `export_head(root, dest, index) -> Result<String, GitError>`; `strip_control` becomes `pub(crate)` | `init`, `export_head` | LRN-12, 15 |
| `src/store.rs` | factor `write_atomic`; add `progress_path`, `load_progress`, `save_progress` | as named | LRN-23, 24 |
| `src/learn.rs` | new; `Nav`, `Action`, `handle_key`, `draw`, report-line builders `quiz_lines`, `check_lines` | as named | LRN-09, 10, 11, 17, 19, 20, 22 |
| `src/ui.rs` | `l` key, dispatch on `Nav`, badges row, job polling, scaffold and save actions, cancel on quit; `View` gains `badges` | `View.badges` | LRN-09, 21, 27 |
| `curriculum/**` | starter, hidden tests, reference solutions | | LRN-04, 12, 15, 17 |
| `tests/academy_*.rs` | new integration tests (see Test plan) | | all |

## Security and scale notes
- Grading runs the learner's own code (build scripts, tests) on their machine; that is the point of the feature. The project path comes from the learner's own `progress.json`. Hidden tests call only library functions and never spawn processes, so the test binary is the only process the timeout must kill.
- Killing a timed-out compile kills `cargo` only; `rustc` may run on until it finishes (TerminateProcess does not end child processes). The reader threads are detached so the UI never waits on them. This is a candidate known limitation.
- Output is capped at 256 KiB per stream and at 40 lines per message. Control characters and ANSI sequences are stripped before drawing.
- The export reads only the HEAD tree. The learner's index, refs and working tree are never written (private `GIT_INDEX_FILE`). The pet's sync stays incremental and unchanged.
- The `<academy>/target` cache grows with the project (a crate with no dependencies, so megabytes). It is kept so reruns are fast.

## Risks and open questions
- Clippy drift: a future toolchain may add warn-by-default lints that flag the reference solutions or the starter, failing LRN-04.a in our CI and failing learners. Mitigation: solutions stay plain and idiomatic; a failing reference test points straight at the lint.
- libtest's pretty output format is not a documented stable interface (`--format json` is unstable). Mitigation: parse only `test NAME ... ok|FAILED` and the `---- NAME stdout ----` blocks. A check name with no `ok` line counts as failed, so a format change fails closed, never open.
- Test suite time: `tests/academy_reference.rs` and `tests/academy_grade.rs` run real cargo builds (about 60 small compiles). Steps run in parallel with separate target dirs. Expect a few minutes; tests are not `#[ignore]`d, because C3 must cover LRN-04.a.
- Edition 2024 in the starter requires a Rust ≥ 1.85 toolchain for the learner. The brief says to `rustup update stable`.
- Line endings: `checkout-index` honors `core.autocrlf`, so the export matches what the learner's own checkout would hold. rustfmt's default `newline_style = Auto` accepts CRLF files (from the rustfmt docs; not verified locally).
- I could not run cargo in this planning sandbox: the rustup proxy reported "rustc.exe ... not applicable". The cargo-side behavior (JSON artifacts, test binary run) is therefore from the docs, not a local run. The git export was verified locally.

## Research sources
- https://git-scm.com/docs/git: "GIT_INDEX_FILE ... specifies an alternate index file"; `--work-tree=<path>` sets the working tree.
- https://git-scm.com/docs/git-read-tree: reads a tree into the index "but does not actually update any of the files"; the work tree is untouched without `-m -u`.
- https://git-scm.com/docs/git-checkout-index: `-a` checks out all index files, `-f` overwrites, and files go to the working directory.
- Local experiment (git 2.45.1.windows.1, path with a space): the temp-index export holds only committed content, and the source repo's `git status` is unchanged.
- https://doc.rust-lang.org/cargo/reference/external-tools.html: `compiler-artifact` has `executable`, `profile.test`, `target.kind` (`["test"]` for integration tests); lines may be non-JSON, so parse only lines starting with `{`; test JSON output is unstable and nightly-only.
- https://doc.rust-lang.org/rustc/json.html: diagnostics have `message`, `code.code` (the lint name), `level` and `rendered`.
- https://doc.rust-lang.org/rustc/tests/index.html: tests run as threads in one process; `--test-threads`, `--color never`; `--format json` is unstable; the pretty format prints `test NAME ... ok`.
- https://doc.rust-lang.org/std/process/struct.Child.html: `try_wait` does not block; `kill` forces exit; a `Child` has no `Drop`, so it must be waited.
- https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess: terminates "the specified process and all of its threads". Child processes are not mentioned; that they survive is general Windows knowledge, unverified on that page.
- https://docs.rs/ratatui/0.30.0/ratatui/backend/struct.TestBackend.html: `TestBackend::new(w, h)` and `buffer()`, with no feature flag, for rendering tests.
- https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html: `std::env::set_var` is unsafe in edition 2024, so tests inject the cargo program name instead (unverified fetch; well known).

## Test plan
| Acceptance criterion | Test (file::name) | Kind |
|----------------------|-------------------|------|
| LRN-01.a | `src/learn.rs::tests::lrn_01_a_lists_all_modules_in_order` (TestBackend render) | unit |
| LRN-02.a | `tests/academy_content.rs::lrn_02_a_every_module_complete` | integration |
| LRN-03.a | `tests/academy_content.rs::lrn_03_a_every_topic_has_objective` | integration |
| LRN-04.a | `tests/academy_reference.rs::lrn_04_a_reference_solutions` | integration (cargo) |
| LRN-05.a | `tests/academy_content.rs::lrn_05_a_objectives_name_nonempty_sections` | integration |
| LRN-06.a | `tests/academy_content.rs::lrn_06_a_items_taught_in_lessons` (also: references are Rust Book URLs only) | integration |
| LRN-07.a | `tests/academy_content.rs::lrn_07_a_tags_known_and_not_later` | integration |
| LRN-08.a | `tests/academy_content.rs::lrn_08_a_every_objective_assessed` | integration |
| LRN-09.a | `src/learn.rs::tests::lrn_09_a_l_opens_list_esc_returns` | unit |
| LRN-10.a | `src/learn.rs::tests::lrn_10_a_lesson_brief_quiz_check_reachable` | unit |
| LRN-11.a | `src/learn.rs::tests::lrn_11_a_module_2_locked` + `progress::tests::lrn_11_open_rule` | unit |
| LRN-12.a | `tests/academy_project.rs::lrn_12_a_scaffold_is_cargo_git` + `lrn_12_a_project_is_watched` | integration |
| LRN-12.b | `tests/academy_project.rs::lrn_12_b_project_commit_feeds_pet` | integration |
| LRN-13.a | `tests/academy_project.rs::lrn_13_a_non_empty_folder_refused_unchanged` | integration |
| LRN-14.a | `src/academy/progress.rs::tests::lrn_14_a_both_parts_needed` | unit |
| LRN-15.a | `tests/academy_grade.rs::lrn_15_a_export_ignores_worktree` + `lrn_15_a_grades_commit_not_worktree` | integration (git, cargo) |
| LRN-15.b | `tests/academy_grade.rs::lrn_15_b_clippy_warning_names_lint` | integration (cargo) |
| LRN-16.a | `tests/academy_grade.rs::lrn_16_a_module_3_names_broken_module_1_check`; `curriculum::tests::lrn_16_checks_upto_includes_earlier` | integration + unit |
| LRN-17.a | `tests/academy_grade.rs::lrn_17_a_failure_report_hides_source` + `src/learn.rs::tests::lrn_17_a_check_lines_show_name_message_section` | integration + unit |
| LRN-18.a | `src/academy/quiz.rs::tests::lrn_18_a_eight_of_ten_passes_seven_fails`; content test that each module has both item kinds | unit |
| LRN-19.a | `src/learn.rs::tests::lrn_19_a_result_names_missed_objective_and_section` | unit |
| LRN-20.a | `src/academy/quiz.rs::tests::lrn_20_a_retake_order_differs` + `learn::tests::lrn_20_a_retake_after_fail` | unit |
| LRN-21.a | `src/ui.rs::tests::lrn_21_a_badge_shown_pet_unchanged` (TestBackend) | unit |
| LRN-22.a | `src/academy/progress.rs::tests::lrn_22_a_failed_retake_keeps_pass` + `src/learn.rs::tests::lrn_22_a_passed_module_reopens` | unit |
| LRN-23.a | `tests/academy_progress.rs::lrn_23_a_restart_restores_progress` | integration |
| LRN-24.a | `tests/academy_progress.rs::lrn_24_a_corrupt_progress_backed_up` (+ missing file case) | integration |
| LRN-25.a | `tests/academy_grade.rs::lrn_25_a_missing_cargo_named` (cargo program `gitagochi-no-such-cargo`; also asserts lesson and quiz data are usable) | integration |
| LRN-26.a | `tests/academy_grade.rs::lrn_26_a_plain_folder_not_cargo` (+ missing folder, no commits) | integration |
| LRN-27.a | `tests/academy_grade.rs::lrn_27_a_compile_error_shown`, `lrn_27_a_infinite_loop_times_out`, `lrn_27_a_spawn_does_not_block`; `proc::tests::lrn_27_timeout_kills` (child `git hash-object --stdin` with stdin held open) | integration + unit |

## Verification
The constitution's quality gates (C3), plus a manual check on Windows:
1. `cargo run`, press `l`. Ten modules are listed, and only module 1 is open.
2. Press `p` and set up a project in a new folder whose path has a space. Commit `count_lines` from that folder; the pet is fed within 10 s.
3. Run the project check while the pet panel keeps ticking. It passes once `cargo fmt` and `clippy` are clean.
4. Take the quiz at ≥ 80%. Module 2 opens, and the pet screen shows a `[1]` badge.
5. Commit an infinite loop and rerun the check. It times out, and the UI keeps responding throughout.
6. Quit with `q` while a check is running. The terminal is restored, and Task Manager shows no `academy_m01*.exe` left behind.

## Analysis
- Requirements without a task: none. LRN-01..27 each map to tasks in `tasks.md`; the per-module content tasks (T013, T023, T024, T026-T031, T033) cover LRN-01..08, LRN-16 and LRN-18 for each module.
- Requirements without a test: none. LRN-04 is covered beyond LRN-04.a only by content review ("each step adds a feature that uses its module's objectives"). LRN-09's "pet keeps updating while the learn screen is open" is covered by the unchanged event loop and manual check 3, with no automated test.
- Tasks without a requirement: none. T001 (types) serves LRN-01, LRN-02, LRN-07 and LRN-16; T034-T035 (docs, gates) serve LRN-09, LRN-12, LRN-23, LRN-25 and C3.
- Test-order exceptions:
  - LRN-16.a needs module 3's content, so its test (T025) comes after the code that implements inheritance (T015). T014 adds a unit test of `checks_upto` with synthetic modules before T015.
  - The LRN-03.a test (T032) comes just before module 10 (T033) instead of in T002, so it is not red for the whole build.
- Spec ambiguities and interpretations, none blocking, to confirm with the owner when convenient:
  1. LRN-12 "add it to the watched repos": the project path is watched on every run through `progress.json`, on top of the CLI or config paths. It is not written into `repos.json`, which would change MR-02's current-directory default. If the owner wants it listed in `repos.json` too, that is a small later addition.
  2. LRN-15 plus LRN-16: module 1 (tooling) has fmt and clippy checks, and later modules inherit earlier checks, so fmt and clippy gate every module, not only modules 1 and 10.
  3. LRN-14: the quiz and the project part may pass at different times. Each part is sticky (best quiz score kept, project pass never cleared).
  4. LRN-11: a locked module cannot be opened at all, lesson included, not only its assessment.
  5. LRN-12: the folder is chosen by typing a path into a prompt. The "no typed free-text answers" non-goal is read as applying to quiz answers only.
  6. LRN-20: "different order" holds within a session. The first attempt after a restart is freshly shuffled but not compared with the last session's order.
- Constitution conflicts: none.
- Candidate known limitations for the close-out:
  - timed-out compiles may leave `rustc` running on Windows;
  - clippy lint drift across toolchains;
  - the learner must keep the crate name `tracker` and the lib+bin layout.
- Result: READY
