---
name: handoff
description: Create a structured handoff document at context/handoff.md at the end of a session so the next agent can onboard without prior context. Uses git to discover modified files, filters out ephemeral subagent work artifacts under context/, enforces a strict format, and validates the result with a lint script. The handoff is the single source of truth for the next session. Use whenever a session ends, a task is paused, or work is being passed to another agent.
---

# Handoff

This skill produces a single, self-contained Markdown document that lets a fresh agent (or human) pick up the work without needing prior context, conversation history, or this skill itself loaded.

The output is always written to `context/handoff.md` relative to the project root.

## The source-of-truth rule

`context/handoff.md` is the **only** durable artifact this skill produces. Everything else under `context/` — `context/plan.md`, `context/research.md`, `context/notes.md`, inter-subagent coordination files, intermediate task lists — is **ephemeral**. It belongs to the current session, may be wiped between sessions, and **must not be relied on by the next agent**.

Consequences for how you write the handoff:

- **Do not link or reference ephemeral `context/*` files** as if the next agent could open them. They may not exist.
- **Distill** the useful parts of those files (key findings from research, surviving items from the plan, decisions taken, dead ends explored) **directly into the relevant sections of the handoff**.
- The only files under `context/` that may be referenced are other handoffs (`context/handoff*.md`), because those follow the same persistence contract.
- If a subagent produced a long artifact (e.g. a survey of 12 libraries) and inlining all of it would bloat the handoff, summarize the conclusion and the rejected options with one-line reasons. Do not write "see `context/research.md`".

The linter warns on `context/*.md` references that aren't handoffs. Treat the warning as a smell, not a false positive.

## When to use

- A session is ending and work is not finished.
- A task is being paused and may be resumed later by a different agent.
- A long-running task is reaching a natural checkpoint and the context window is getting tight.
- The user explicitly asks for a handoff.

Do not use this skill for trivial one-off tasks where there is nothing meaningful to hand off.

## Inputs you must gather

Before writing the document, collect the following. Do not guess. If something is unknown, mark it explicitly in the document.

1. **Project root**: `git rev-parse --show-toplevel`. Abort with an explanation if not inside a git repo.
2. **Current branch and HEAD**: `git rev-parse --abbrev-ref HEAD` and `git rev-parse --short HEAD`.
3. **Commits created during the session**: list new commits on the current branch that did not exist at the session start. If you do not have a reliable session start ref, ask the user for one or fall back to commits authored today by the current user (`git log --since="$(date -I) 00:00" --author="$(git config user.email)" --pretty=format:'%h %s'`). State explicitly which strategy you used.
4. **Modified files (full picture, including work done by sub-agents)**:
   - Staged + unstaged: `git status --porcelain`
   - Committed in session: `git diff --name-status <session-start-ref>..HEAD`
   - Untracked: included in `git status --porcelain` as `??`
   - Merge the three lists, deduplicate by path, keep the most relevant status per file.
5. **Ephemeral subagent artifacts to distill**: read every file under `context/` (except `context/handoff*.md`), extract the durable content, and fold it into the appropriate handoff sections (Context & Background, Decisions & Rationale, Open Points, Problems & Caveats, References). Then ignore those files for the "Edited Files" listing.
6. **Task summary, status, open points, problems, next steps**: derive from the session itself. Be precise. No filler.
7. **Skills the next agent should load**: list by skill name, with one sentence per skill explaining why it is relevant.

## Filtering work/helper files

Sub-agents and orchestration produce throwaway files (plans, scratchpads, inter-agent messages, research dumps). These must not appear under "Edited Files". Apply this filter in order:

1. **Hard ignore by path pattern** (always treated as ephemeral, never listed):
   - **Everything under `context/` except `context/handoff*.md`** — this is where pi-subagents drop their session-local state.
   - `.work/`, `work/`, `scratch/`, `.scratch/`, `tmp/`, `.tmp/`, `.agent/`, `agent-comm/`, `.handoff-tmp/`
   - Files matching `*.scratch.md`, `*.tmp`, `*.work.md`, `*.agent.md`
2. **Gitignored files**: run `git check-ignore -v <path>` for each candidate. If ignored, treat as ephemeral unless it is a handoff file.
3. **Heuristic by filename at repo root or top-level docs**: `PLAN.md`, `TODO.md`, `NOTES.md`, `AGENT_NOTES.md`, `subagent-*.md`, `coordination.md`. Treat as ephemeral unless they were already tracked in git before the session started (`git log --oneline -- <path>` returns commits older than session start).
4. **Ambiguous cases** (real work that just *looks* like a helper): list them under "Possible work artifacts (please confirm)" inside "Edited Files" instead of silently dropping them. Never delete files. Never `git add` or `git rm` anything as part of the handoff.

If you are uncertain whether a file is real work or a work artifact, keep it visible to the user. Silent omission is worse than a noisy entry.

## Producing the document

1. Read `TEMPLATE.md` from this skill directory.
2. Before filling sections, read all `context/*.md` files that aren't handoffs and absorb their content. Then act as if those files no longer exist.
3. Fill in every section. If a section legitimately has no content, write `_None._` (with the underscores) so the linter can distinguish "empty on purpose" from "forgotten".
4. Keep the section order and heading text exactly as in the template. The linter is strict about this.
5. Write the result to `<project-root>/context/handoff.md`. Create the `context/` directory if it does not exist.
6. Do not commit the handoff. Leave that decision to the user.
7. Do **not** delete the ephemeral `context/*` files yourself. The harness/user manages their lifecycle.

## Validation

After writing the file, run the linter:

```sh
bash <skill-dir>/lint.sh <project-root>/context/handoff.md
```

Exit code `0` means valid. Any non-zero exit means the document must be fixed before ending the session. Do not silently ignore lint errors. If a rule is wrong for the situation, explain it in the handoff and to the user; do not paper over it.

Warnings are not failures but should be addressed. In particular, a warning about `context/*.md` references almost always means you forgot to distill an ephemeral artifact.

## Style rules for the handoff content

- Write in English regardless of the conversation language.
- Be specific. Replace vague phrases ("did some refactoring") with concrete ones ("extracted `parseConfig` from `cli.ts` into `config/parse.ts`, kept behavior identical, added 4 unit tests").
- Prefer lists over prose for files, commits, steps, and risks.
- Quote exact commands, file paths, env vars, and error messages verbatim in backticks.
- Never invent commits, files, or test results. If you did not verify something, say so.
- The document must make sense to an agent that has never heard of this skill. Do not reference "the handoff skill" inside the document.

## Self-contained output

The handoff document is the contract. Anything the next agent needs to continue must be either in the document or reachable via a path/link the document provides — and that link must point to something that will still exist (committed code, tracked docs, external URLs, prior handoffs). Assume zero prior context and zero ephemeral state.
