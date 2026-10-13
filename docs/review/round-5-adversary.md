# Round 5 adversary (commit efc1d20)

Reran build, clippy -D warnings, fmt --check and cargo test myself: all pass (every test binary ok).

## Reviewer claims
- Round-4 items 1-3 met: CONFIRMED. Run end to end (repos.json temporarily placed in the real data dir %APPDATA%\gitagochi, then removed; `dirs` ignores an APPDATA override, so isolation was not possible):
  - BOM + single backslash, no args: `error: ...\repos.json: invalid repos.json: invalid escape at line 1 column 15`, exit 1, before any terminal setup.
  - BOM + doubled backslashes: parses to the right path (checked via `load_config` from a scratch crate).
- "Untested `run` guard does not matter": OVERSTATED, but still minor. It is the round-4 blocker's main fix and has no regression test. I ran it by hand and it works. Nothing blocks, but the guard sits in `run` with no seam, so a future refactor can silently revert it. Suggest extracting a pure fn (cli_paths_empty, config_warning) -> Result and unit-testing it.
- "Un-launched TUI does not matter": CONFIRMED for the `extend` line (it compiles, and the app launched fine with a valid config in my runs). I could not confirm on screen that the warning stays visible after a rescan with CLI paths: the TUI output is cell-addressed escape codes, so grep cannot find the text. Unverified by run; the logic at src/ui.rs:165 and :178 reads correctly.
- Cosmetic "doubled prefix": WRONG wording. The message is `<path>: invalid repos.json: ...`, which is redundant but not doubled. Trivial.
- Redundant `msgs.extend(config_warning)` at lib.rs:136: CONFIRMED.

## Missed
### MINOR M1: PowerShell 5 files are usually UTF-16, not UTF-8 with BOM
`>` and `Out-File` in Windows PowerShell 5 write UTF-16LE with a BOM, and `Set-Content` writes ANSI. Only `-Encoding UTF8` gives the UTF-8 BOM that this round handles. I tested both through `load_config`:
- UTF-16LE (`ff fe ...`) and ANSI (`caf\xe9`) both give `cannot read repos.json: stream did not contain valid UTF-8`.
- Run for real, the UTF-16 file now exits 1 with `error: ...\repos.json: cannot read repos.json: stream did not contain valid UTF-8`.

This is no longer a silent fallback, which was the round-4 concern. But the message does not tell the user to re-save as UTF-8, and the README says nothing about encoding. Suggested: add the hint "save as UTF-8" to the read-error text and one README sentence. Not blocking.

### MINOR M2: fatal with CLI paths absent also hides a working default
A user with a broken repos.json cannot run in a repo directory without passing `.` explicitly. This is the intended behavior per the verdict, and the error names the file. Only worth documenting in the README.

## Verdict on this round
No BLOCKING findings. Reviewer's conclusion stands. M1 and the missing guard test are minor and non-blocking.
