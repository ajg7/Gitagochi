# NNN <Feature name>: tasks

Work top to bottom. Each task is small enough for one commit, leaves the quality gates green, and names the requirements it serves. The coder ticks a box in the same commit that finishes the task. `[P]` marks tasks that don't depend on the one before them.

## Phase 1: Foundations
- [ ] T001 <task> — files: `src/...` — serves: XX-01
- [ ] T002 [P] <task> — files: `src/...` — serves: XX-02

## Phase 2: Behavior
- [ ] T003 Write failing test for XX-01.a — files: `tests/...`
- [ ] T004 Make XX-01.a pass — files: `src/...` — serves: XX-01

## Phase 3: Polish
- [ ] T00N Update README/CLAUDE.md usage if user-visible behavior changed
- [ ] T00N Run all quality gates and launch the app
