# Gitagochi
A terminal based tomagachi-like game that feeds on git commits

## Meet your Gitagochi
Your pet grows as it eats your commits: egg, then baby, then adult. Here it is in a happy mood at each stage.

```
   ___       .---.     /\___/\
  / . \     ( ^.^ )   ( ^   ^ )
 |  .  |     ( v )     (  w  )
  \___/      '---'     /|   |\
                        '---'
  Egg        Baby        Adult
```

It also gets hungry, sleepy or content depending on how recently you committed.

## About this project
This was my first experiment in AI agentic workflows, such as an orchestrator-workers setup combined with an evaluator-optimizer loop. A main Claude Code session acted as the orchestrator and wrote no code itself. It handed work to specialised subagents (coder, reviewer, adversary, gatekeeper and tester), defined in `.claude/agents/`. The agents passed state to each other through files in `docs/review/` rather than chat.

Each round worked like this:
1. The coder implements the spec (`docs/spec.md`) or fixes the items from the last verdict.
2. The reviewer reviews the diff.
3. The adversary critiques the review and the code, looking for what the reviewer missed or overstated.
4. The gatekeeper runs the build, clippy, fmt and tests, then returns `SATISFIED` or `REVISE`.
5. Once satisfied, the tester writes tests from the spec alone and flags any blocking bugs, which feed into the next round.

The loop was capped at three rounds. If it still ended in `REVISE` after the cap, the orchestrator would stop and escalate to me. The review, adversary and verdict files from each round are kept in `docs/review/`.

## Usage
```
cargo run -- [PATH] [--author EMAIL]
```
Keys: `q`, `Esc` or `Ctrl-C` quits, and `r` refreshes.
