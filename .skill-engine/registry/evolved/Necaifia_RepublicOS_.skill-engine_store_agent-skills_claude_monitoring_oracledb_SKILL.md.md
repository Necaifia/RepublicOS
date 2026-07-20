---
description: Use these skills to manage and monitor Oracle databases by executing
  SQL statements, exploring schema metadata, analyzing query performance, monitoring
  active sessions and resource consumption, and managing storage and object health.
  Implements trace-level observability with OpenTelemetry GenAI conventions, eval-driven
  monitoring, and cost attribution.
metadata:
  evolved: true
  evolved_at: '2026-07-20T14:07:04.946843+00:00'
name: oracledb
tags:
- monitoring
- database
version: 2
---

## Usage

All scripts can be executed using Node.js. Replace `<param_name>` and `<param_value>` with actual values.

**Bash:**
`node <skill_dir>/scripts/<script_name>.js '{"<param_name>": "<param_value>"}'`

**PowerShell:**
`node <skill_dir>/scripts/<script_name>.js '{\"<param_name>\": \"<param_value>\"}'`

Note: The scripts automatically load the environment variables from various .env files. Do not ask the user to set vars unless skill executions fails due to env var absence.


## Scripts


### execute_sql

Executes any SQL statement.

#### Parameters

| Name | Type | Description | Required | Default |
| :--- | :--- | :--- | :--- | :--- |
| sql | string | The SQL to execute. | Yes |  |


---

### get_query_plan

Generate a full execution plan for a single SQL statement using EXPLAIN PLAN. This can be used to analyze query performance without execution. Requires the SQL statement as input as a parameter.

#### Parameters

| Name | Type | Description | Required | Default |
| :--- | :--- | :--- | :--- | :--- |
| query | string | The SQL statement for which you want to generate plan (omit the EXPLAIN keyword). | Yes |  |


---

### list_active_sessions

List the top N (default 50) currently running database sessions (STATUS='ACTIVE'), showing SID, OS User, Program, and the current SQL statement text.



---

### list_invalid_objects

Lists all database objects that are in an invalid state, requiring recompilation (e.g., procedures, functions, views).



---

### list_tables

Lists all user tables in the connected schema, including segment size, row count, and last analyzed date. Filters by a comma-separated list of names. If names are omitted, lists all tables in the current user's schema.



---

### list_tablespace_usage

List tablespace names, total size, free space, and used percentage to monitor storage utilization.



---

### list_top_sql_by_resource

List the top N (default 5) SQL statements from the library cache based on a chosen resource metric (CPU, I/O, or Elapsed Time). Shows SQL ID, execution count, buffer gets, disk reads, CPU time, and elapsed time.



---

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


## Security

- Never hardcode secrets, tokens, or credentials in skill files or scripts
- Use environment variables or secret management tools for sensitive values
- Validate all user inputs before processing
- Follow least-privilege principle: request only the permissions you need
- Log all security-relevant actions for audit


## Configuration

- Use environment variables with sensible defaults for configuration
- Validate configuration at the start of execution
- Document all configuration options and their effects
- Support loading config from files when appropriate
