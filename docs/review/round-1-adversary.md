# Round 1 adversary

Read all of src/ against docs/spec.md. Built the project; the TUI was not driven interactively.

## Verdicts on reviewer claims

1. ui.rs:113 byte slice panic: CONFIRMED in code (`&c.id[..len.min(7)]` panics on a non-char boundary). Only reachable via a hand-edited save, so MINOR is right.
2. git.rs:114 branch switch refeeds: CONFIRMED by reasoning, and I rate it UNDERSTATED. Scenario: feed commits on branch X (last_seen = X tip), checkout other branch Y (last_seen = Y tip), checkout X again. `Y..X` contains every X commit not reachable from Y, including ones already fed, so they feed again (hunger/health/total_fed inflated, up to 1000 per switch). Switching branches is routine, and the spec says "each new commit ... since the last seen commit". I would call it BLOCKING-leaning but it is arguably not a literal spec violation, so I leave it as MINOR with strong recommendation: skip ids already in `recent`, or keep a bounded seen-set. (I only verified the git range semantics with a scratch repo; I did not run the app end to end.)
3. git.rs:133 any `git log` failure treated as a rebase and silently re-baselined: CONFIRMED, MINOR. Note it also means a transient failure permanently skips the pending commits.
4. "Save only after a poll ... lose feed state": WRONG / non-finding. The text contradicts itself. `refresh` sets `dirty` and the loop saves in the same iteration, before the next draw, so last_seen and fed state are persisted together. Only decay-only ticks are unsaved, and those are recomputed from `last_update`. No action needed; it should not be in the list.
5. Unbounded summary: CONFIRMED, MINOR (git subjects are one line, so realistically a long line gets clipped by the List widget; no crash).
6. Missing Mood tests: CONFIRMED. Spec only demands tests for feeding, decay, stage transitions, clock edge cases, save/load, all present, so MINOR.

Reviewer claim "terminal restore on Ctrl-C": CONFIRMED for raw mode (handled as a key event). Not covered: Ctrl-C delivered while a `git log` child runs is still a key event queued, fine. OK.

## Missed by the reviewer (all MINOR)

A. store.rs:27 `to_lowercase()` on the repo path for the save key applies on every platform; on case-sensitive filesystems `/x/Repo` and `/x/repo` share one save file. Harmless on Windows (the target).
B. lib.rs:31/ui.rs `refresh`: when `sync` returns an error (e.g. git vanished mid-run) the message is shown but nothing else retries differently; acceptable. Not a bug, noted only.
C. A fresh pet created from a corrupt save re-baselines at current HEAD, so commits made while the app was closed and the save was corrupt are dropped. Consistent with the spec ("fresh pet") so not a defect.
D. store.rs `load`: a save missing any field (e.g. after a future version adds a field) fails deserialization and is moved to `.bak`, resetting the pet. No `#[serde(default)]` on `Pet`. MINOR forward-compat note.
E. `--author` matches the author, not committer, and the match is a case-insensitive substring of `<email>`; fine per spec.
F. Tests gap: no test exercises `git.rs` against a real repo (empty repo, non-git dir, rebased baseline, author filter) or `sync` end to end. Spec lists these as required handled cases; they are implemented but untested beyond `parse_log`. MINOR, worth adding as an integration test in tests/ (the reviewer says 3 integration tests exist; I did not verify which behaviors they cover).

## Spec gaps

None blocking found. Non-git dir, empty repo (last_seen None then `HEAD` walk bounded to 1000), large history (range plus max-count), corrupt/missing save with `.bak`, backwards clock, and no Unix-only APIs all check out in the code.

## Conclusion

No new BLOCKING issues. Reviewer's list is mostly accurate; item 4 is a non-finding and item 2 is understated (recommend fixing it this round via dedupe against `recent`).
