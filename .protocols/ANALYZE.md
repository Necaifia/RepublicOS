# Protocol: Analyze

**Phase:** 1 of the engineering cycle

## Steps

1. Read `.memory/PROJECT_STATE.json` — understand current phase.
2. Read `.memory/TASK_QUEUE.json` — see what's queued and in progress.
3. Read `.memory/HANDOFF.md` — pick up from last session.
4. Read `.constitution/SUCCESS.md` — verify mission is not already complete.
5. If `phase == "init"`: do full repository reconnaissance (file tree, language, dependencies).
6. List findings in a structured summary.

## Output

- Understanding of current state
- Confirmation the previous task is actually complete
- No duplicate work detected
