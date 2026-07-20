# Security Gate

Pre-merge checks:

```
[ ] Secrets scan             — no API keys, tokens, passwords in diff
[ ] Dependency audit         — no known vulnerable dependencies added
[ ] Unsafe review            — all unsafe {} blocks justified
[ ] Credentials              — no hardcoded credentials
[ ] Input validation         — all user inputs sanitized
[ ] Output encoding          — no XSS or injection vectors
```

If any fails, the Security Officer blocks the merge.
