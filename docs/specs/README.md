# Specs

Each feature lives in its own folder, numbered in the order it was started:

```
docs/
  constitution.md            rules for every feature
  specs/
    _templates/              copy these for a new feature
    001-core-pet/
      spec.md                what and why (owner approves)
      plan.md                how (planner)
      tasks.md               ordered, checkable steps (planner; coder ticks them)
      review/                round-N-review / -adversary / -verdict, tester-report
    002-multi-repo/
      ...
  review/                    archive of the reviews written before this layout
```

## Lifecycle

| Step | Who | Output | Gate |
|------|-----|--------|------|
| 1. Specify | you + main session (`/specify`) | `spec.md`, `Status: Draft` | every requirement has an ID and acceptance criteria |
| 2. Clarify | you + main session (`/specify` again) | `[NEEDS CLARIFICATION]` markers resolved into the Clarifications log | no markers left |
| 3. Approve | you | `Status: Approved` | you say so |
| 4. Plan | `planner` | `plan.md` | follows the constitution |
| 5. Tasks | `planner` | `tasks.md` | each task names the requirement IDs it serves |
| 6. Analyze | `planner` | "Analysis" section at the end of `plan.md` | every requirement has a task and a test; no task without a requirement; no constitution conflicts |
| 7. Implement | `coder` → `reviewer` → `adversary` → `gatekeeper` (max 3 rounds) | code, ticked `tasks.md`, `review/round-N-*.md` | `VERDICT: SATISFIED` |
| 8. Verify | `tester` | tests per acceptance criterion, `review/tester-report.md` with a coverage table | no BLOCKING bugs |
| 9. Close | main session | `Status: Implemented`, known limitations added to `spec.md` | spec matches what shipped |

Status values: `Draft` → `Approved` → `In progress` → `Implemented` (or `Superseded by NNN`).

## Writing requirements

Use EARS phrasing so each line is testable:

- Always: `The app shall <response>.`
- Event: `When <trigger>, the app shall <response>.`
- State: `While <state>, the app shall <response>.`
- Unwanted: `If <bad condition>, then the app shall <response>.`
- Optional: `Where <feature is enabled>, the app shall <response>.`

Give each acceptance criterion as Given/When/Then. Mark anything you are not sure about with `[NEEDS CLARIFICATION: question]` rather than guessing.

## Changing a shipped feature

Small change to existing behavior: edit that feature's `spec.md` (bump its version, add to its changelog), set `Status: Approved` again once you sign off, then plan only the delta. New capability: start a new numbered folder.
