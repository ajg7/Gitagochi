# Round 6 review (commit 1692984)

Checks run by me: clippy -D warnings clean, fmt --check clean, all tests pass (0 ignored). The tester BLOCKING item (unreadable HEAD clobbering last_seen, causing refeed on recovery) is fixed.

No BLOCKING findings.

Behavior verified by reading the code:
- Empty repo with no commits: first sync stores last_seen None, so had_baseline is false and no warning fires. Correct.
- Freshly added repo: first sync records baseline and returns early; head is Some, so no warning.
- Recovered repo: last_seen was kept, so commits_since(old) returns only commits after the baseline. No refeed. New commits feed. Rewritten history while down is handled by the existing invalid-since path.
- A vanished directory makes head() return None without an Err, so sync_repo returns Ok(0) and the warning path is the one that reports it.
- The warning repeats on every poll while the repo stays unavailable. It does not accumulate, because `message` is rebuilt each refresh. This looks intended (a persistent status). It does outrank config warnings (`errors.first()` wins).

MINOR
1. src/lib.rs:71-75: the unavailable check calls `r.head()` again, an extra git subprocess per repo per poll. `commits_since` already calls head() too, so the poll does it twice, plus the check is a possible TOCTOU if HEAD changes between calls. Suggested fix: have `sync_repo` return the poll head or a small enum (Fed(n), Unavailable) so sync_all does not re-query.
2. src/lib.rs:75: an unresolvable HEAD on a healthy repo (for example after `git checkout --orphan`, before the first commit) also shows "repo unavailable" on every poll. The text is slightly misleading. Suggested wording: "HEAD unreadable". Not worth blocking.
3. Only an integration test covers the baseline fix (tests/multi_repo_spec.rs:328). There is no test asserting the "repo unavailable" error appears, and none asserting it does not appear for an empty repo. Add two small tests.
