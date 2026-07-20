# Quality Gate

Every cycle must pass ALL of the following. If any fails, fix and retry.

```
[ ] Build                    — project compiles without errors
[ ] Tests                    — full test suite passes
[ ] Lint                     — linter / formatter passes
[ ] No warnings              — zero compiler or linter warnings
[ ] Security                 — no secrets, no vulnerable dependencies
[ ] Coverage                 — new code has tests
[ ] Documentation            — README, docs, comments current
[ ] State updated            — all .memory/ files current
[ ] Next Action generated    — NEXT_ACTION.md written
[ ] Handoff written          — HANDOFF.md complete
[ ] Quality Gate logged      — result recorded in METRICS.json
```

## Failure handling

1. Note which gate failed.
2. Fix the issue.
3. Re-run all gates from the top.
4. No exceptions for "small changes."
