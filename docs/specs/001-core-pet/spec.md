# 001 Core pet

Status: Implemented
Version: 1.0
Depends on: none
Requirement prefix: CORE

Backfilled on 2026-11-08 from the original `docs/spec.md` (v1). Built in review rounds 1-3; those review files are archived in `docs/review/`. No `plan.md` or `tasks.md` were written for this feature.

## Why
Coding alone gives little day-to-day feedback. A terminal pet that lives on your commits turns committing into a small habit loop: commit and it thrives, stop and it gets hungry.

## User stories
- As a developer, I want a pet in my terminal that I feed by committing, so that I'm nudged to commit regularly.
- As a developer, I want the pet to keep living while the app is closed, so that it reflects my real habits, not just when I watch it.

## Requirements

### Feeding
- **CORE-01** When the configured author makes a new commit after the last seen commit, the app shall feed the pet once for that commit.
- **CORE-02** The app shall use `git config user.email` as the author unless one is given.
- **CORE-03** The app shall feed equally for every commit, with no message or diff analysis.
- **CORE-04** When a repo is seen for the first time, the app shall record its HEAD as the baseline and feed nothing from existing history.
- **CORE-05** The app shall feed each commit at most once, including after branch switches or rewritten history.

### Pet rules
- **CORE-06** While time passes, the app shall raise hunger; while hungry, the app shall lower health.
- **CORE-07** When the pet is fed, the app shall lower hunger and raise health.
- **CORE-08** The app shall derive the stage (egg, baby, adult) from total commits fed.
- **CORE-09** The app shall derive the mood (happy, hungry, sleepy) from hunger and health.
- **CORE-10** If the pet starves, then the app shall make it sleepy rather than dead, and feeding shall recover it.

### UI
- **CORE-11** The app shall show a full-screen TUI with the pet in ASCII, hunger and health bars, recently fed commits and key hints.
- **CORE-12** When the user presses `q`, `Esc` or `Ctrl-C`, the app shall quit; when the user presses `r`, it shall check for commits immediately.
- **CORE-13** While running, the app shall check for new commits periodically.
- **CORE-14** When the app quits, panics or is interrupted, the app shall restore the terminal.

### Persistence
- **CORE-15** The app shall save pet state as JSON in the platform data dir.
- **CORE-16** If the save file is missing, then the app shall start a fresh pet.
- **CORE-17** If the save file is corrupt, then the app shall keep it as `.bak`, start a fresh pet and not crash.
- **CORE-18** When the app starts, the app shall apply the decay for the time it was closed.
- **CORE-19** If the clock goes backwards, then the app shall not panic and shall not apply negative decay.

### Errors and scale
- **CORE-20** If the path is not a git repo, then the app shall exit with a clear message and exit code 1.
- **CORE-21** If the repo has no commits, then the app shall run without error and feed once commits appear.
- **CORE-22** The app shall read only new commits, so start-up time does not grow with history size.

## Acceptance criteria
- **CORE-01.a** Given a baselined repo, when the author makes 2 commits and the app syncs, then total fed rises by 2.
- **CORE-01.b** Given a baselined repo, when someone else commits, then nothing is fed.
- **CORE-04.a** Given a repo with 10 existing commits and no save, when the app syncs, then nothing is fed and HEAD is stored as the baseline.
- **CORE-05.a** Given a fed commit, when the user switches branches and back, then that commit is not fed again.
- **CORE-06.a** Given hunger 0, when 8 hours pass, then hunger is about 100.
- **CORE-06.b** Given hunger below 60, when time passes but hunger stays below 60, then health does not change.
- **CORE-07.a** Given hunger 50 and health 50, when one commit is fed, then hunger is 25 and health is 55.
- **CORE-08.a** Given 0, 3 and 20 commits fed, then the stage is egg, baby and adult respectively.
- **CORE-09.a** Hunger ≥ 90 or health ≤ 25 is sleepy; otherwise hunger ≥ 60 is hungry; otherwise happy.
- **CORE-10.a** Given a sleepy, starved pet, when enough commits are fed, then it is no longer sleepy.
- **CORE-14.a** Given the TUI is running, when the process panics, then raw mode and the alternate screen are left.
- **CORE-16.a** Given no save file, when the app starts, then a fresh egg exists.
- **CORE-17.a** Given a save file containing invalid JSON, when the app starts, then a `.bak` file exists and a fresh pet is used.
- **CORE-18.a** Given a save from 4 hours ago, when the app loads it, then hunger has risen by about 50.
- **CORE-19.a** Given a save timestamped in the future, when the app ticks, then nothing panics and hunger does not drop.
- **CORE-20.a** Given a plain directory, when the app runs on it, then it prints an error and exits 1.
- **CORE-21.a** Given an empty repo, when a first commit is made and the app syncs, then the app does not error.
- **CORE-22.a** Given a repo with more than 1000 new commits, when the app syncs, then at most 1000 are read in that sync.

## Tuning (as built)
Hunger fills from 0 to 100 in about 8 hours. Health drains from 100 to 0 in about 12 hours while hunger is ≥ 60. Each commit removes 25 hunger and adds 5 health. Baby at 3 commits, adult at 20. Poll every 10 seconds. 20 recent commits shown; commit summaries cut to 200 characters; up to 10,000 fed ids kept per repo.

## Non-goals
Multiple pets, networking, sound, config UI, commit content analysis.

## Clarifications
- 2026-11-08: The v1 draft's "(assumption)" items (author default, egg/baby/adult, no death, equal feeding, `q`/`r` keys, periodic polling) shipped as written and are now requirements CORE-02, -03, -08, -10, -12, -13.

## Known limitations
- Single-repo, one-file-per-repo saves were replaced by one shared save in 002.

## Changelog
- 1.0 (2026-11-08): backfilled from `docs/spec.md` with requirement IDs and acceptance criteria.
