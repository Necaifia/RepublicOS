# Pantheon

**A cell runtime for autonomous software pipelines.**

> Everything is a Cell. Humans, AI, schedulers, HTTP servers, filesystems — all cells.

```
174 tests  ·  0 warnings  ·  45 Rust files  ·  7,500+ LOC  ·  11 crates
Demo: 524µs  ·  1000-cell simulation: 95ms  ·  Deterministic replay
```

```bash
pantheon run                    # Run the pipeline demo
pantheon run --with-human       # Human-as-a-cell (asks for approval)
pantheon init my-project        # Create a new pipeline project
pantheon dashboard 8080         # Live event timeline (Wireshark-style)
pantheon replay events.log      # Frame-by-frame time travel
pantheon verify events.log      # Deterministic verification
```

---

## What is Pantheon?

Pantheon is a **distributed runtime where every component is a Cell**.

Cells communicate via **capability-guaranteed events**, not APIs.
Tasks are scheduled by **priority** with five levels (Critical → Background).
Everything — humans, AI models, schedulers, plugins — is a Cell.

The engine is **AI-neutral**. AI providers (OpenAI, Claude, Gemini, DeepSeek) are just plugins.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Pantheon Engine                    │
├──────────┬──────────┬──────────┬──────────┬─────────┤
│  Kernel  │  Caps    │  Events  │ Scheduler│ Metrics │
│  (types) │ (access) │  (bus)   │ (tasks)  │ (stats) │
├──────────┴──────────┴──────────┴──────────┴─────────┤
│  Runtime · Memory · Plugins (trait + SDK)            │
├─────────────────────────────────────────────────────┤
│  CLI: demo · simulate · run · init · visualize      │
│  Tools: doctor · inspect · dashboard · replay       │
│         snapshot · restore · verify                  │
└─────────────────────────────────────────────────────┘
```

### Kernel
- **Cell**: unique ID (ULID), status, role. Everything is a Cell.
- **Capability**: `CapabilityGrant` with attenuation chains. `issuer → subject` with `parent` tracking.
- **Event**: Lamport-timestamped messages with capability verification and TTL.
- **ProtocolRef**: versioned protocol references for cross-cell contracts.

### Capabilities (7 tests)
- `CapabilityManager`: issue, delegate, verify, revoke, check chain, list.
- Attenuation narrows resource, actions, or constraints.
- Delegation sets `issuer = self.subject` for correct chain semantics.

### Event Bus (17 tests)
- `EventBus`: multi-threaded publish/subscribe/dispatch.
- `Partition`: logical split with leader/follower.
- `EventLog`: append, read, compact, filter.

### Scheduler (29 tests)
- Priority queue (BinaryHeap): Critical → High → Normal → Low → Background.
- FIFO within same priority. Three task kinds: OneShot, Recurring, EventDriven.
- Built-in metrics: pending, completed, cancelled.

### Metrics (11 tests)
- Counters, gauges, histograms (p50/p95/p99).
- JSON export, formatted display.

### Plugins

```rust
pub trait PantheonPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn init(&mut self, ctx: PluginContext) -> Result<()>;
    fn on_start(&self) -> Result<()>;
    fn on_event(&self, event: &Event) -> Result<Option<Event>>;
    fn on_task(&self, task: &ScheduledTask) -> Result<TaskAction>;
    fn on_shutdown(&self) -> Result<()>;
}
```

Implement in <30 min. Three built-in examples:
- **EchoPlugin** — logs every event (minimal template)
- **CounterPlugin** — counts events via `MemoryManager` (stateful)
- **FilterPlugin** — cancels matching tasks (task interception)

**LocalHumanPlugin** — human-as-a-cell. Pipeline pauses, asks terminal, resumes.

---

## Quick Start

```bash
# Clone and run
git clone https://github.com/your-org/pantheon
cd pantheon
cargo run -- run

# See the cell pipeline in action:
#   Planner → Backend → Test → Review → Release
#   Live progress bars, event timeline, ASCII graph

# Run with human approval gate
cargo run -- run --with-human

# Create your own project
cargo run -- init my-pipeline
cd my-pipeline
# Edit pantheon.toml, then:
cargo run -- run

# Launch the dashboard
cargo run -- dashboard 8080
# Open http://localhost:8080

# Run the full test suite
cargo test
```

---

## CLI Reference

```
Pipeline:
  init [name]            Create a new pipeline project
  run                    Run pipeline demo
    --with-human         Enable human-as-a-cell plugin

System:
  demo                   End-to-end system demonstration
  simulate [N]           N-cell simulation (default: 1000)
  visualize              Live ASCII cell graph

Debug:
  doctor                 Run 7 health checks
  inspect                Live system state
  dashboard [port]       Web dashboard (default: 8080)

Tools:
  replay [file]          Replay event log (-i for interactive)
  snapshot [file]        Save state (.ptn)
  restore [file]         Restore state from .ptn
  verify [file]          Deterministic verification
```

---

## Numbers

| Metric | Value |
|---|---|
| Rust files | 45 |
| Lines of code | 7,500+ |
| Cargo crates | 11 |
| Tests | 174 |
| Failures | 0 |
| Warnings | 0 |
| Demo duration | 524 µs |
| 1000-cell simulation | 95 ms |
| Property tests | 13 |
| Pipeline stages | 5 |
| CLI commands | 11 |
| Health checks | 7 |

---

## Roadmap

- [x] **Foundation**: Kernel types, capability grants, Lamport clocks
- [x] **Engine**: Capability manager, event bus, scheduler, metrics
- [x] **Tools**: CLI, dashboard, replay, snapshot, verify, doctor, inspect
- [x] **Plugins**: Trait + LocalHumanPlugin (human-as-a-cell)
- [x] **Runtime**: Cell lifecycle, task execution engine (11 tests)
- [x] **Memory**: Persistent key-value store, snapshots (14 tests)
- [x] **Plugin SDK**: 3 examples (Echo, Counter, Filter), Memory access, full docs
- [x] **Property tests**: 13 proptests for kernel, clock, resources, capabilities
- [x] **Mutation config**: `mutants.toml` for cargo-mutants baseline
- [ ] **Core plugins**: Git, Filesystem, Timer, HTTP
- [ ] **AI plugins**: OpenAI, Claude, Gemini, DeepSeek

---

## Why "Everything is a Cell"?

Because services, humans, and AI shouldn't be different concepts.

- A **Human Cell** asks the terminal for decisions.
- An **AI Cell** asks a language model.
- A **Git Cell** runs `git` commands.
- An **HTTP Cell** serves requests.

All cells speak the same protocol: **capability-guaranteed events**.  
The engine doesn't care what's on the other end — it just routes, schedules, and verifies.

This is what makes Pantheon **AI-neutral** and **infrastructure-agnostic**.

---

## License

Pantheon Community License v1.0
