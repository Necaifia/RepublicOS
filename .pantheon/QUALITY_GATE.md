# Quality Gate

Every cycle must pass ALL of the following before it is considered complete.

If any gate fails, the cycle is rejected. Fix the failure and retry.

```
[ ] Build                   — cargo build --workspace
[ ] Tests                   — cargo test --workspace
[ ] Benchmarks              — cargo bench (no regressions)
[ ] Formatting              — cargo fmt --check
[ ] Clippy                  — cargo clippy --workspace -- -D warnings
[ ] No warnings             — zero compiler warnings
[ ] Documentation updated   — README, docs, inline comments current
[ ] State updated           — TASK_HISTORY.json, PROJECT_STATE.json, METRICS.json
[ ] Next Action generated   — NEXT_ACTION.md written
[ ] Known Issues reviewed   — KNOWN_ISSUES.md checked for new entries
[ ] Quality Gate logged     — Log the cycle result in METRICS.json
```

## Failure handling

1. Note which gate failed.
2. Fix the issue.
3. Re-run all gates from the top.
4. Do NOT skip gates because "it's a small change."
