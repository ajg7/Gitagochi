# 003 Rust academy

Status: In progress
Version: 1.2
Depends on: 001, 002 (the learner's project is a watched repo; its commits feed the pet under CORE-01/MR-11, unchanged); LRN-29 narrows MR-09
Requirement prefix: LRN

## Why
Gitagochi users write code and commit, and many want to learn Rust itself. A built-in, project-based Rust course lets a programmer who is new to Rust learn by building one real project in their own repo, so studying and feeding the pet are the same activity. Every assessment checks only what the curriculum taught, so a pass means the learner can do what the module set out to teach.

## User stories
- As a programmer new to Rust, I want guided modules that each grow one project, so that I learn by building instead of only reading.
- As a learner, I want each module's assessment to test what that module taught, so that a pass means I actually learned it.
- As a learner, I want my project graded automatically with useful feedback, so that I know what to fix and where to relearn it.
- As a learner, I want my progress saved, so that I can stop and resume across sessions.
- As a learner, I want my project commits to feed my pet and my passes to show on its screen, so that studying keeps it alive and visible.

## Requirements
### Curriculum
- **LRN-01** The app shall ship a built-in, ordered Rust curriculum that works offline, written for programmers who know another language and git but are new to Rust.
- **LRN-02** Each module shall state its learning objectives, give an in-app lesson, set a project step, and end with an assessment.
- **LRN-03** The curriculum shall cover at least: cargo and tooling (build, test, fmt, clippy); types, variables and control flow; ownership and borrowing; structs, enums and pattern matching; error handling with `Result` and `?`; collections and iterators; traits and generics; modules, visibility and testing; lifetimes; and idiomatic-Rust best practices.
- **LRN-04** The project steps shall together build one driving project: a command-line commit tracker that reads a log of commits and reports stats (such as counts per author and streaks), ending with a tiny pet of the learner's own. Each step shall add a feature that uses its module's objectives.
- **LRN-05** Every objective shall be taught in a named section of its module's lesson; the assessment shall test only objectives that a lesson section teaches.
- **LRN-06** Each lesson may list "read more" references to the official Rust Book; no assessment item shall depend on content found only in a reference.

### Alignment of assessments to content
- **LRN-07** Every assessment item (quiz question or project check) shall be tagged with the objective(s) it assesses; every tag shall name an objective of that module or an earlier one.
- **LRN-08** Every objective of a module shall be assessed by at least one item in that module's assessment.

### Learn screen
- **LRN-09** When the user presses `l` on the pet screen, the app shall open a learn screen showing the module list with locked, open and passed states; a key shall return to the pet screen, and the pet shall keep updating while the learn screen is open.
- **LRN-10** From the learn screen, the learner shall be able to read a module's objectives and lesson, read its project brief, take its quiz, and run its project check.
- **LRN-11** While a module is not passed, the app shall keep later modules locked.

### Project setup
- **LRN-12** When the learner chooses an empty or new folder to start the project, the app shall create the starter cargo project there, initialise it as a git repo, and add it to the watched repos so its commits feed the pet.
- **LRN-13** If the chosen folder exists and is not empty, then the app shall refuse and change nothing.

### Assessment
- **LRN-14** Each module's assessment shall have two parts, a project part and a quiz part; the module is passed only when both pass.
- **LRN-15** The project part shall grade the project's last commit, not uncommitted changes, by running the module's checks (tests the learner cannot see, plus `cargo fmt --check` and `cargo clippy -D warnings` for the modules on tooling and best practices); it passes only when every check passes.
- **LRN-16** The project checks of module N shall include the checks of every earlier module, so later steps cannot break earlier features.
- **LRN-17** When a project check fails, the app shall show each failing check's name, the compiler, test or lint message, and the objective and lesson section to revisit; it shall not show the hidden test source.
- **LRN-18** The quiz shall contain multiple-choice and spot-the-bug (pick the correct fix) items, and shall pass at a score of 80% or more.
- **LRN-19** When the learner submits a quiz, the app shall show the score and, for each missed item, the objective it tested and the lesson section that teaches it.
- **LRN-20** The app shall let the learner retake a quiz or rerun a project check any number of times; a retake shall present the items in a different order, and every attempt shall present each item's choices in a shuffled order.
- **LRN-21** When a module is passed, the app shall show a badge for it on the pet screen; passing shall not change hunger, health or feeding (CORE-03 still holds).
- **LRN-22** While a module is passed, the learner shall be able to reread it and retake its assessment for practice without losing the pass.

### Progress
- **LRN-23** The app shall save curriculum progress (passed modules, best quiz scores, project folder) in the platform data dir and restore it on start.
- **LRN-24** If the progress file is missing, then the app shall start at module 1; if it is corrupt, then the app shall keep it as `.bak`, start at module 1 and not crash.

### Errors
- **LRN-25** If the Rust toolchain is missing, then the app shall say which tool is missing and how to install it, and keep lessons and quizzes usable.
- **LRN-26** If the project folder is missing, is not a cargo project, or has no commits, then the app shall say which and not pass the project part.
- **LRN-27** If the project does not build or a check runs too long, then the app shall fail that check with the build output or a timeout message and stay responsive.
- **LRN-28** The project part's result shall depend only on whether the checks really passed; nothing the learner's code does (printing fake results, exiting early) shall make a check pass.
- **LRN-29** If no paths are given and no repo can be resolved (no config, and the current directory is not or holds no repo), then the app shall open with a "no repos watched yet" notice instead of exiting, so the learn screen and project setup are reachable. Explicit paths that resolve to no repo still exit 1 (CORE-20, MR-09).

## Acceptance criteria
- **LRN-01.a** Given no network, when the learner opens the learn screen, then all modules are listed in order.
- **LRN-02.a** Given any module, when it is opened, then its objectives, lesson, project brief and assessment are all present.
- **LRN-03.a** Given the shipped curriculum, then each topic listed in LRN-03 is the subject of at least one objective.
- **LRN-04.a** Given a reference solution for each step, when module N's checks run against step N's solution, then they pass; against step N-1's solution, then at least one of module N's own checks fails.
- **LRN-05.a** Given the shipped curriculum, then every objective names a lesson section of its own module and that section exists and is non-empty.
- **LRN-06.a** Given the shipped curriculum, then every item's tagged objectives are taught in a lesson section (not only in a reference).
- **LRN-07.a** Given the shipped curriculum, when every item's tags are checked, then none is untagged and none names an unknown or later objective.
- **LRN-08.a** Given the shipped curriculum, then each objective of each module is tagged by at least one item of that module.
- **LRN-09.a** Given the pet screen, when `l` is pressed, then the module list shows; when the back key is pressed, then the pet screen returns.
- **LRN-10.a** Given an open module, when the learner selects lesson, brief, quiz and check in turn, then each is reachable.
- **LRN-11.a** Given module 1 not passed, when the learner tries to open module 2's assessment, then it is locked.
- **LRN-12.a** Given a new folder, when the learner starts the project there, then it contains a cargo project and a git repo, and it appears in the watched repos.
- **LRN-12.b** Given that project, when the learner commits to it, then the pet is fed as for any watched repo.
- **LRN-13.a** Given a non-empty folder, when the learner starts the project there, then the app refuses and the folder is unchanged.
- **LRN-14.a** Given a passed quiz and a failed project part, then the module is not passed; given both passed, then it is.
- **LRN-15.a** Given a committed solution and an uncommitted change that breaks it, when the check runs, then it passes (the commit is graded).
- **LRN-15.b** Given a tooling or best-practices module and a committed solution with a clippy warning, when the check runs, then the project part fails naming the lint.
- **LRN-16.a** Given module 3's step that breaks a module 1 feature, when module 3's check runs, then it fails naming the module 1 check.
- **LRN-17.a** Given a failing check, then the failing check's name, its message and the objective and lesson section are shown, and no hidden test source is shown.
- **LRN-18.a** Given a quiz of 10 items, when 8 are correct, then it passes; when 7 are correct, then it fails.
- **LRN-19.a** Given a quiz with one wrong answer, when submitted, then the score is shown and the missed item's objective and lesson section are named.
- **LRN-20.a** Given a failed quiz, when retaken, then it can be taken again and the item order differs.
- **LRN-20.b** Given any module's quiz, when every attempt's answers always pick the same displayed position (1st, 2nd, ...), then no such strategy reaches the 80% pass mark with certainty; across attempts the correct choice's position varies.
- **LRN-21.a** Given module 1 just passed, when the pet screen shows, then a module 1 badge is visible and hunger and health are unchanged by the pass.
- **LRN-22.a** Given a passed module, when its quiz is retaken and failed, then the module is still passed.
- **LRN-23.a** Given module 1 passed and a project folder set, when the app restarts, then module 1 shows as passed, module 2 is open and the project folder is remembered.
- **LRN-24.a** Given a progress file of invalid JSON, when the app starts, then a `.bak` exists and progress starts at module 1.
- **LRN-25.a** Given no Rust toolchain on PATH, when a project check runs, then a message names the missing tool and lessons and quizzes still work.
- **LRN-26.a** Given a project folder that is a plain folder, when checked, then the app says it is not a cargo project and the project part is not passed.
- **LRN-27.a** Given a committed project that does not compile, when checked, then the check fails with the compiler output and the UI keeps responding.
- **LRN-28.a** Given a committed project that fails a hidden check but whose code prints fake passing test lines (e.g. `test m01_x ... ok`, `test result: ok`), when checked, then that check still fails.
- **LRN-28.b** Given a committed project whose code calls `std::process::exit(0)` inside the function under test, when checked, then that check fails.
- **LRN-29.a** Given no `repos.json` and a current directory that holds no repo, when the app starts with no paths, then it opens, shows the notice, and `l` opens the learn screen.
- **LRN-29.b** Given an explicit path that is a plain folder, when the app runs on it, then it still exits 1.

## Edge cases
- Progress file missing / corrupt → LRN-24
- Toolchain missing → LRN-25
- Project folder missing, not cargo, no commits → LRN-26
- Project fails to build, or a check hangs (infinite loop) → LRN-27
- Learner's uncommitted edits → ignored, LRN-15
- Chosen folder not empty → LRN-13
- Later step breaks an earlier feature → LRN-16
- Mis-tagged or untaught assessment item → caught at test time by LRN-05..08
- Windows paths with spaces → LRN-12, LRN-15 (C5)

## Non-goals
- Not a general Rust reference; teaches only what the curriculum lists.
- Not for people new to programming.
- No network, accounts, leaderboards, AI grading or typed free-text answers.
- No in-app code editor; the learner uses their own editor and git.
- No progress reset; passed modules stay passed.
- Passing does not feed the pet or change pet rules.

## Open questions

## Clarifications
- 2026-11-08: Q: Project shape? A: one driving project across modules → LRN-04, LRN-16
- 2026-11-08: Q: How is the project part checked? A: hidden tests plus fmt/clippy gates; quiz pass mark 80% → LRN-15, LRN-18
- 2026-11-08: Q: Where does learning happen? A: a learn screen inside the TUI → LRN-09, LRN-10
- 2026-11-08: Q: Target learner? A: knows another language and git, new to Rust → LRN-01
- 2026-11-08: Q: Pet tie-in? A: commits feed as usual; passes show badges only → LRN-21
- 2026-11-08: Q: Driving project? A: mini commit-tracker CLI ending in a tiny pet → LRN-04
- 2026-11-08: Q: Project setup? A: app scaffolds the starter project and auto-watches it → LRN-12, LRN-13
- 2026-11-08: Q: What is graded? A: the last commit, not the working tree → LRN-15
- 2026-11-08: Q: Quiz item types? A: multiple choice and spot-the-bug → LRN-18
- 2026-11-08: Q: Feedback on failure? A: failing check names, messages and lesson pointer; test source hidden → LRN-17
- 2026-11-08: Q: Lesson content? A: self-contained in-app, optional Rust Book pointers → LRN-05, LRN-06
- 2026-11-08: Q: Reset? A: revisit and retake freely, no reset → LRN-22
- 2026-11-08: Q: Planner interpretations 1-6 in plan.md Analysis (project watched via progress file not repos.json; fmt/clippy carried to every module; quiz and project parts pass independently; locked modules fully closed; folder chosen by typed path; retake order differs within a session)? A: accepted as planned → LRN-12, LRN-15, LRN-16, LRN-14, LRN-11, LRN-20
- 2026-11-08: Q (round 1 gap S2): must answer choices be shuffled? A: yes, every attempt → LRN-20, LRN-20.b
- 2026-11-08: Q (round 1 gap S1): new user with no repo cannot reach the learn screen? A: open with no repos and a notice, when no paths are given → LRN-29 (narrows MR-09)
- 2026-11-08: Q (round 1 gap S3): must grading resist fake test output printed by learner code? A: yes → LRN-28

## Known limitations
- A determined learner can still cheat by inspecting the compiled hidden test program (LRN-28); grading resists accidents and casual tricks, not a motivated attacker.

## Changelog
- 0.1 (2026-11-08): first draft.
- 0.2 (2026-11-08): open questions resolved with the owner.
- 1.0 (2026-11-08): approved by the owner.
- 1.1 (2026-11-08): owner decisions on round 1 spec gaps: shuffled choices (LRN-20), LRN-28, LRN-29.
- 1.2 (2026-11-09): LRN-28 covers early exits too; added LRN-28.b and a known limitation (owner approved round 4).
