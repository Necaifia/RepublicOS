---
description: Analyze queried data for trends, week-over-week comparisons, distributions,
  funnels, cohorts, top-N lists, anomalies, sanity checks, and report-ready findings.
  Use after or alongside ClickHouse queries when the user wants insight rather than
  raw rows. Follows current best practices for reliability, security, and maintainability.
  Enforces least-privilege, secret management, input validation, and audit logging
  throughout.
metadata:
  evolved: true
  evolved_at: '2026-07-20T14:07:05.462416+00:00'
name: analyzer
tags: []
version: 2
---

# Analyzer

Turn data into defensible findings instead of only returning rows.

## Analysis patterns

Choose the smallest pattern that answers the question:

- Trend: metric over time at the right grain.
- Comparison: current period vs prior period, release vs baseline, or segment A vs B.
- Distribution: percentiles, skew, tails, and outliers.
- Funnel: step counts, conversion rates, and drop-offs.
- Cohort: behavior grouped by start date, version, source, or first action.
- Top-N: largest contributors with share of total.
- Sanity check: row counts, null rates, first/last seen, duplicates, and data freshness.

## Before concluding

- Verify the time window and grain match the user's question.
- Check sample size, nulls, and whether the metric is dominated by a small tail.
- Look for freshness, rollout, telemetry opt-in, or version-coverage issues.
- Avoid causal language unless the query design supports causality.
- If the result is surprising, run or propose one validation query before presenting it as fact.

## Finding format

```md
Finding: ...
Evidence: ...
Confidence: High/Medium/Low because ...
Caveats: ...
Recommended next check: ...
```

## Prerequisites

- Ensure all required tools and dependencies are installed
- Verify you have the necessary permissions and access credentials
- Check that the target environment is in a known good state


## Error Handling

- Always check the exit code or response status of commands before proceeding
- On failure, log the error details and attempt recovery if a retry strategy exists
- If recovery fails, report the error with context: what was attempted, what went wrong, and suggested next steps
- Never silently ignore errors — treat unexpected output as potential failure


## Verification

- After each step, verify the expected outcome before continuing
- Use idempotent checks: running the same action twice produces the same result
- If verification fails, roll back the last change and report the issue
- Log verification results for audit trail


## Configuration

- Use environment variables with sensible defaults for configuration
- Validate configuration at the start of execution
- Document all configuration options and their effects
- Support loading config from files when appropriate
