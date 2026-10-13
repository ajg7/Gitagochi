VERDICT: REVISE
ESCALATE: no

Evidence: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test` all pass (every test binary ok). The app launches and draws the TUI ("Gitagochi - watching 1 repo"). The only blocker is the repos.json handling, which I confirmed from the reviewer and adversary notes and the README text (README.md:36 still has single backslashes).

## Required (BLOCKING)

1. README.md:36: the repos.json example is invalid JSON (`\U` is not a valid escape). Change it to doubled backslashes (`"C:\\Users\\me\\Tech & Coding"`) or forward slashes. Add one sentence saying that Windows backslashes must be doubled. Expected outcome: a user who copies the example gets a config that `parse_config` accepts.

2. src/repos.rs:29-40 (`parse_config` / `load_config`), src/lib.rs `run`, src/ui.rs `refresh`: an unparseable repos.json must not silently turn into "watch `.`".
   - Expected outcome: when no CLI paths were given and repos.json exists but fails to parse, `run` exits 1 with a message that includes the file path and serde's line/column.
   - If CLI paths were given, the config is not needed, so a warning is enough. That warning must stay visible. Either keep `session.initial_warning` through the rescans (src/ui.rs `warnings = r.warnings;`), or show it once on the CLI path.
   - Add a unit test that a single-backslash path makes `parse_config` return an error.

3. src/repos.rs `parse_config`: strip a leading `\u{feff}` (BOM) before parsing (adversary N1). Windows PowerShell and Notepad write a BOM, and it triggers the same failure as item 2. Add a unit test that a BOM-prefixed config parses.

I made item 3 required because it is the same silent-fallback bug on the platform the user runs, and the fix is one line.

## Deferred (minor)

- Polling and scanning run on the UI thread. About 12 repos gives a brief stall every 10s. Move them to a worker thread later.
- Hidden or system directory attributes on Windows are not checked in the scan (reviewer 3).
- Case-insensitive de-dup (reviewer 4).
- `resolve` for a subfolder of a repo that contains nested repos (reviewer 5).
- Document in the README that the tracking of dropped legacy repos is lost (reviewer 6).
- Restore the `Pet::feed` assertion that `recent` is capped at `MAX_RECENT` (reviewer 7).
- `refresh` shows only the first error or warning (reviewer 8).
- Cosmetic: redundant `.git` in `SKIP_DIRS`, and the literal newline in `msgs.join` (reviewer 9).
- N2: one author is applied to all repos. Document that `author` should be set when per-repo emails differ.
- N3: a repo that is dropped and re-added feeds its backlog. Document or prune.
- N4: pet.json is rewritten every poll even when nothing changed.
- N5: the `[repo]` tag is the folder name only, so same-named repos look identical.
- N6: repos.json is read once at startup. Document this.
