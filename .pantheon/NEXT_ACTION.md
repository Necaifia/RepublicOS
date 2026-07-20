# Next Action

## Task: REPL mode (from queue: "repl")

Enhance the CLI with:
- Command history (up/down arrow support via rustyline or manual stdin)
- Better error messages with context
- Graceful handling of EOF (Ctrl+D)
- Persistent history across sessions (optional)

## Success criteria
- REPL starts with `cargo run`
- `2 + 3` prints `= 5`
- Division by zero prints clear error
- `quit` / `exit` / Ctrl+D exits cleanly
- All 17 existing tests still pass

## Quality Gate
Pass all 11 checks before committing.
