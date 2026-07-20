# Protocol: Handoff

**Phase:** Final phase of every cycle

## Steps

1. Read all current `.memory/` files.
2. Update `.memory/TASK_HISTORY.json` — add the completed task.
3. Update `.memory/PROJECT_STATE.json` — new phase, test counts, summary.
4. Update `.memory/HANDOFF.md` — what was done, what's next, blockers.
5. Update `.memory/NEXT_ACTION.md` — the single next step.
6. Update `.memory/METRICS.json` — increment cycles, log quality gate result.
7. Update `.constitution/SUCCESS.md` — check off completed boxes.
8. If `.constitution/SUCCESS.md` has zero unchecked boxes:
   - Set `"done": true` in `.memory/PROJECT_STATE.json`
   - Write `.memory/COMPLETION_REPORT.md`
   - Stop

## Rules

- Every cycle ends with a handoff. No exceptions.
- If the cycle failed, document why in HANDOFF.md.
- The next session reads HANDOFF.md to continue instantly.
