# Round 6 adversary (commit 1692984)

Reran: cargo build OK, clippy -D warnings clean, fmt --check clean, cargo test all pass (0 ignored). Real data dir untouched (scratch crate in temp dir, tempfile repos, sync_all only).

## Reviewer claims
- Tester BLOCKING (unreadable HEAD clobbers last_seen) fixed: CONFIRMED. `last_seen` is only overwritten when `poll.head.is_some()`; test at tests/multi_repo_spec.rs:328 passes.
- Empty repo gives no warning: CONFIRMED (scratch C1/C2 errs=[], first commit later feeds 1).
- Vanished dir gives Ok(0) plus "repo unavailable": CONFIRMED (B1).
- Warning outranks config warnings: CONFIRMED (ui.rs:241 `errors.first()` wins, config warning only shown in the else branch). It also hides a real sync error from a later repo, since only the first error is shown. Not new.
- MINOR 1 (extra git call): CONFIRMED, slightly understated: it is 3 `rev-parse HEAD` per repo per poll when healthy... actually 2 (commits_since + check) for a healthy repo; TOCTOU is harmless.
- MINOR 2 (orphan checkout wording): CONFIRMED by code reading, not run.
- MINOR 3 (no test for the warning): CONFIRMED.

## Missed
1. MINOR (arguably same class as the fixed bug, narrower trigger): a repo that is unreadable at its FIRST sync is baselined as `initialized=true, last_seen=None`. When it comes back, `commits_since(None)` treats it as an empty repo and feeds all history (up to 1000 commits). Repro (scratch crate against the library): repo with 5 commits, directory renamed away, `sync_all` (fed=0, no error, no warning because had_baseline is false), directory renamed back, `sync_all`:
   `A2 after recovery fed=5 errs=[]`
   This violates "existing history does not feed" in the spec. The trigger needs a transient failure on the very first poll (repo on a flaky/network drive, git lock, removed between resolve and first sync), so it is rare; I rate it MINOR, but it is the same refeed failure the tester flagged. Fix: in sync_repo's first-sync branch, if `repo.head()` is None and the repo root is not a dir / `rev-parse --git-dir` fails, do not set `initialized`; or store a distinct "empty" marker versus "unreadable".
2. MINOR: the "repo unavailable" warning is not emitted on the first-sync unavailable case (same cause), so the user gets no signal.
3. Delete and re-create with unrelated history (B): while gone, warning shows; after re-creation `fed=0` with silent re-baseline to the new HEAD (old last_seen invalid, existing invalid-since path). No refeed, no warning. Acceptable; new commits made between re-creation and the next poll are lost, which is inherent to the rewritten-history path.
4. HEAD moved while unavailable: the same as normal polling; commits since the old baseline feed once, deduplicated by fed_ids. No issue found.
5. Not checked: terminal cleanup, Windows path quirks beyond rename on Windows (worked).

## Verdict input
No new BLOCKING. The tester's BLOCKING item is genuinely fixed for the stated scenario. Item 1 is MINOR and may be accepted as a known limitation or fixed cheaply.
