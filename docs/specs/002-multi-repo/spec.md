# 002 Multi-repo

Status: Implemented
Version: 1.1
Depends on: 001 (supersedes its one-save-per-repo rule, CORE-15)
Requirement prefix: MR

Backfilled on 2026-11-08 from the "Multi-repo (v1.1)" section of the original `docs/spec.md`. Built in review rounds 4-6; those review files are archived in `docs/review/`.

## Why
Real work is spread across many repos. A pet fed by only one of them goes hungry on days spent elsewhere, so it stops reflecting how much you actually commit.

## User stories
- As a developer with many repos, I want one pet fed by all of them, so that every commit counts.
- As a developer, I want to point the app at a parent folder, so that new repos under it are picked up without editing config.
- As an existing user, I want my current pet kept when I upgrade, so that I don't lose its progress.

## Requirements

### Selecting repos
- **MR-01** The app shall accept several paths on the command line.
- **MR-02** When no paths are given, the app shall read `repos` and `author` from `repos.json` in the data dir; when there is neither, it shall use the current directory.
- **MR-03** The app shall let command-line paths override the config's `repos`, and `--author` override its `author`.
- **MR-04** If `repos.json` cannot be parsed and no path is given, then the app shall exit with an error naming the file, line and column.
- **MR-05** When a path is not itself a repo, the app shall scan below it for repos up to 3 levels deep.
- **MR-06** While scanning, the app shall not descend into a repo, hidden directories, `node_modules`, `target`, `venv`, `.venv` or `__pycache__`, and shall not follow symlinks or junctions.
- **MR-07** While running, the app shall re-scan at least once a minute and when the user presses `r`, so new repos are picked up.

### Errors
- **MR-08** If a configured path is missing or not a repo, then the app shall skip it, show a one-line warning, and keep feeding from the others.
- **MR-09** If no repo at all can be resolved, then the app shall exit with a clear message and exit code 1. (Narrowed by LRN-29: when no paths are given, the app opens with a notice instead.)
- **MR-10** If a watched repo becomes unreadable, then the app shall warn, keep its baseline, and resume without refeeding when it recovers.

### One shared pet
- **MR-11** The app shall feed one pet from all selected repos and keep it in one save file, whichever repos are selected on a given run.
- **MR-12** The app shall track the last seen commit and fed commits per repo, keyed by canonical repo path.
- **MR-13** When a repo is seen for the first time, the app shall baseline only that repo (CORE-04 applies per repo).
- **MR-14** If one repo's history is rewritten or its baseline vanishes, then the app shall re-baseline only that repo.
- **MR-15** The app shall apply the large-history cap (CORE-22) per repo.

### UI
- **MR-16** The app shall show which repo each recent commit came from, and how many repos are watched.

### Migration
- **MR-17** When only old per-repo saves exist, the app shall adopt the most recently modified one as the shared pet, keeping that repo's tracking.

## Acceptance criteria
- **MR-01.a** Given two repos A and B, when run with both paths, then commits in either feed the same pet.
- **MR-02.a** Given `repos.json` listing A and no args, when the app runs, then A is watched.
- **MR-03.a** Given `repos.json` listing A, when run with path B, then only B is watched.
- **MR-04.a** Given an invalid `repos.json` and no args, when the app runs, then it exits 1 with a message containing the file name, line and column.
- **MR-05.a** Given a repo at `root/x/y/repo`, when run on `root`, then the repo is found; at 4 levels deep it is not.
- **MR-06.a** Given a repo with a nested repo inside it, when its parent is scanned, then only the outer repo is watched.
- **MR-06.b** Given a repo inside `node_modules` or a hidden directory, when scanned, then it is not watched.
- **MR-07.a** Given a running app on `root`, when a new repo is created under `root` and `r` is pressed, then it is watched.
- **MR-08.a** Given paths A (valid) and M (missing), when the app runs, then A feeds and a warning names M.
- **MR-09.a** Given only missing paths, when the app runs, then it exits 1.
- **MR-10.a** Given a baselined repo whose HEAD becomes unreadable for one sync, when it recovers, then no old commits are fed.
- **MR-11.a** Given a pet fed from A, when run later with only B, then the same pet (same total fed) is loaded.
- **MR-13.a** Given a watched repo A, when repo B with history is added, then B's history does not feed.
- **MR-14.a** Given repos A and B, when A is force-reset to an unrelated history, then only A is re-baselined and B's tracking is unchanged.
- **MR-16.a** Given commits from A and B, then the recent list labels each with its repo name and the header says "watching 2 repos".
- **MR-17.a** Given two old saves with different modification times and no shared save, when the app starts, then the newer one's pet is used and its repo's tracking is kept.

## Non-goals
Multiple pets, per-repo pets, networking, config UI.

## Known limitations
- If a newly added repo is unreadable on its very first sync, it is stored with no baseline, and on recovery its existing history (up to 1000 commits, deduplicated) feeds the pet. Accepted in round 6; a fix needs an "unreadable" vs "empty" marker in the save format.
- The UI shows only the first warning when several exist.
- An orphan checkout before its first commit shows "repo unavailable" rather than "HEAD unreadable".

## Changelog
- 1.1 (2026-11-08): backfilled from `docs/spec.md` with requirement IDs, acceptance criteria and the known limitations from the round-6 verdict.
