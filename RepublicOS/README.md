# RepublicOS

**Turn any AI into a disciplined engineering organization.**

RepublicOS is a vendor-neutral protocol that transforms any capable AI model
(ChatGPT, Claude, Gemini, OpenCode, Codex, Cursor, Windsurf) into a complete
engineering organization — with roles, protocols, quality gates, and persistent
memory across sessions.

```
.gitignore               ← you already have this
.constitution/           ← who you are, what you value, when you stop
.organization/           ← role contracts with authority, boundaries, outputs
.memory/                 ← persistent state across sessions
.protocols/              ← step-by-step workflows for every engineering activity
.gates/                  ← quality checks that every cycle must pass
.templates/              ← project scaffolds for common types
AI_ENTRYPOINT.md         ← the single file your AI reads to become the org
QUICKSTART.md            ← get started in 60 seconds
```

## Quick start

```bash
# 1. Copy RepublicOS into your project (or just copy the files you need)
cp -r RepublicOS/ your-project/

# 2. Set your mission
#    Edit .constitution/MISSION.md with your goal
#    Edit .constitution/SUCCESS.md with your definition of done

# 3. Run the doctor to verify everything is ready
powershell -File republicos.ps1 doctor   # Windows
./republicos.sh doctor                   # Linux/macOS

# 4. Tell any AI
```

> Analyze this repository.
>
> Read AI_ENTRYPOINT.md first.
>
> Behave according to RepublicOS.

**That's it.** The AI reads the constitution, assigns roles, plans the first
cycle, implements, tests, reviews, and hands off — until your success criteria
are met.

[See the weather-cli validation →](https://github.com/Necaifia/weather-cli)

## Design principles

1. **Vendor-neutral by construction.** The protocol is plain Markdown + JSON.
   No SDK, no API, no dependency on any specific AI provider. Any model that
   reads files can run it.

2. **Session persistence via files, not conversation.** State lives in
   `.memory/`. The next session picks up exactly where the last one stopped —
   even with a different AI model.

3. **Atomic cycles, one task per commit.** Every cycle is a single, testable
   unit of work. No scope creep, no half-baked features.

4. **Quality gates are mandatory.** No exceptions for "small changes." If it
   isn't tested, it isn't done.

5. **No invented work.** If the AI can't access a resource, it blocks and
   reports the blocker. No pretending, no hallucinated results.

6. **Same protocol, any scale.** Level 0 (single AI simulating roles) through
   Level 3 (distributed agents). The files don't change — the AI adapts.

## What RepublicOS is not

- **Not a framework.** You don't install anything. No npm install, no cargo add.
- **Not a prompt.** The constitution replaces the need for long instructions.
- **Not tied to a tool.** Works with ChatGPT, Claude, Gemini, OpenCode, Codex —
   any AI that can read files.

## Levels of autonomy

| Level | Capability | How it runs |
|---|---|---|
| 0 | Single AI | Simulates roles sequentially in one response |
| 1 | Role-aware AI | Assigns itself a role per phase |
| 2 | Native agents | Spawns parallel agents per role |
| 3 | Distributed | Agents run on separate machines |

Same files. Same protocol. The AI adapts automatically to its capability.

## Validated

RepublicOS was validated end-to-end on **weather-cli** — a Rust CLI tool
built across 3 cycles by 2 different AI models (ChatGPT plan + OpenCode
execution) with zero shared context beyond the `.memory/` files.

- 3 engineering cycles
- 13 tests, zero failures
- Zero compiler warnings
- Released as v1.0.0

## License

MIT
