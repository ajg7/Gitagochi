---
name: specify
description: Start or clarify a Gitagochi feature spec. Use when the user describes a new feature or a change to existing behavior, says "/specify", "new spec", "write a spec", or "clarify the spec". Creates or updates docs/specs/NNN-<feature>/spec.md from the template, resolves open questions with the user, and marks it Approved only on the user's say-so. Never writes code.
---

# Specify

Turns a feature idea into an approved `spec.md`. Covers the Specify, Clarify and Approve steps in `docs/specs/README.md`. Read `docs/constitution.md` and that README first.

## 1. Find or create the spec
- Change to a shipped feature: open that feature's `spec.md`, bump its version, set `Status: Draft`, and add a changelog line.
- New capability: pick the next number after the highest folder in `docs/specs/`, create `docs/specs/NNN-<kebab-name>/`, copy `docs/specs/_templates/spec.md` into it, and pick a 2-4 letter requirement prefix not used by another spec.

## 2. Draft
- Fill Why, user stories, requirements (EARS, one ID each), acceptance criteria (Given/When/Then, at least one per requirement), edge cases, non-goals.
- Write *what*, never *how*: no crates, files or function names.
- Read the existing specs so new requirements don't contradict them; when they supersede one, say which ID.
- Anything you'd be guessing becomes `[NEEDS CLARIFICATION: ...]` under Open questions. Don't guess silently.

## 3. Clarify
- Ask the user the open questions one at a time, most impactful first, at most 5 in a session. Offer your recommended answer with each.
- After each answer: update the affected requirements, remove the marker, and add a line to Clarifications.

## 4. Self-check before asking for approval
- Every requirement has an ID and an acceptance criterion; every criterion is observable from outside the code.
- No `[NEEDS CLARIFICATION]` left.
- Nothing conflicts with `docs/constitution.md` or another spec.

## 5. Approve
Show the user the requirements and acceptance criteria. Set `Status: Approved` only when they say so. Then tell them the next step: run the planner on this folder (see CLAUDE.md).
