//! # Pantheon Plugin SDK
//!
//! Build plugins that run inside the Pantheon cell runtime.
//!
//! ## Quick Start — 10-minute plugin
//!
//! ```rust
//! use pantheon_plugin::{PantheonPlugin, PluginContext, TaskAction};
//! use pantheon_kernel::error::Result;
//! use pantheon_kernel::event::Event;
//! use pantheon_scheduler::task::ScheduledTask;
//!
//! pub struct EchoPlugin;
//!
//! impl PantheonPlugin for EchoPlugin {
//!     fn name(&self) -> &'static str { "echo" }
//!     fn description(&self) -> &'static str { "Echoes every event it receives" }
//!
//!     fn init(&mut self, _ctx: PluginContext) -> Result<()> { Ok(()) }
//!
//!     fn on_event(&self, event: &Event) -> Result<Option<Event>> {
//!         println!("  EchoPlugin received: {}", event);
//!         Ok(None) // don't modify, just observe
//!     }
//! }
//! ```
//!
//! ## Lifecycle
//!
//! 1. **`init`** — called once at registration; store the `PluginContext` for later use.
//! 2. **`on_start`** — called when the runtime begins execution.
//! 3. **`on_event`** / **`on_task`** — called for every event/task flowing through the system.
//! 4. **`on_shutdown`** — called when the runtime shuts down.
//!
//! ## Task handling
//!
//! Return `TaskAction::Pass` to let the task continue, `TaskAction::Handled` to consume it,
//! or `TaskAction::Cancel(reason)` to abort it.
//!
//! ## Memory access
//!
//! Use `ctx.memory.lock().unwrap()` to read/write cell memory from `init` or any `&self` method,
//! as long as you stored `ctx` during `init`.

pub mod local_human;

use std::sync::{Arc, Mutex};
use pantheon_kernel::error::Result;
use pantheon_kernel::event::Event;
use pantheon_kernel::cell_id::CellId;
use pantheon_scheduler::task::ScheduledTask;
use pantheon_capabilities::manager::CapabilityManager;
use pantheon_event_bus::bus::EventBus;
use pantheon_scheduler::Scheduler;
use pantheon_metrics::MetricsCollector;
use pantheon_memory::MemoryManager;

/// What a plugin wants to do with a task it inspected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskAction {
    /// Task was handled; do not pass to next plugin.
    Handled,
    /// Let the task continue through the chain.
    Pass,
    /// Cancel the task with a reason.
    Cancel(String),
}

/// Shared access to Pantheon subsystems.
///
/// Store this during [`PantheonPlugin::init`] and use it in your
/// `on_event` / `on_task` / `on_shutdown` methods.
#[derive(Clone)]
pub struct PluginContext {
    pub event_bus: Arc<Mutex<EventBus>>,
    pub scheduler: Arc<Mutex<Scheduler>>,
    pub capabilities: Arc<Mutex<CapabilityManager>>,
    pub metrics: Arc<Mutex<MetricsCollector>>,
    pub memory: Arc<Mutex<MemoryManager>>,
}

/// A plugin that runs inside a Pantheon runtime.
///
/// # Implementing
///
/// You only *must* implement:
/// - [`name`](PantheonPlugin::name)
/// - [`description`](PantheonPlugin::description)
/// - [`init`](PantheonPlugin::init)
///
/// Everything else has a sensible default (no-op / pass-through).
///
/// # Thread safety
///
/// Plugins must be `Send + Sync` because the runtime may call
/// `on_event` / `on_task` from different threads. In practice this
/// means all state lives behind `Arc<Mutex<…>>` or similar.
pub trait PantheonPlugin: Send + Sync {
    /// Short, unique name for this plugin (e.g. `"git-sync"`).
    fn name(&self) -> &'static str;

    /// Human-readable description of what this plugin does.
    fn description(&self) -> &'static str;

    /// Initialise the plugin with runtime access.
    ///
    /// Store `ctx` in your struct so you can emit events, schedule
    /// tasks, write metrics, or read/write memory later.
    fn init(&mut self, ctx: PluginContext) -> Result<()>;

    /// Called once when the runtime starts.
    fn on_start(&self) -> Result<()> {
        Ok(())
    }

    /// Inspect (and optionally replace) an event.
    ///
    /// Return `Ok(Some(event))` to replace the event, or `Ok(None)`
    /// to leave it unchanged.
    fn on_event(&self, event: &Event) -> Result<Option<Event>> {
        let _ = event;
        Ok(None)
    }

    /// Inspect a task and decide its fate.
    ///
    /// Return [`TaskAction::Pass`] to let it continue,
    /// [`TaskAction::Handled`] to consume it, or
    /// [`TaskAction::Cancel`] to abort.
    fn on_task(&self, task: &ScheduledTask) -> Result<TaskAction> {
        let _ = task;
        Ok(TaskAction::Pass)
    }

    /// Called once when the runtime shuts down.
    fn on_shutdown(&self) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Example plugins — full working implementations
// ---------------------------------------------------------------------------

/// An example plugin that logs every event it sees.
///
/// ```
/// use pantheon_plugin::{PantheonPlugin, PluginContext, EchoPlugin};
/// use pantheon_kernel::error::Result;
///
/// let mut p = EchoPlugin::new();
/// assert_eq!(p.name(), "echo");
/// assert!(p.description().len() > 0);
/// ```
pub struct EchoPlugin;

impl EchoPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl PantheonPlugin for EchoPlugin {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Logs all events. Minimal plugin — no context needed."
    }

    fn init(&mut self, _ctx: PluginContext) -> Result<()> {
        Ok(())
    }

    fn on_event(&self, event: &Event) -> Result<Option<Event>> {
        println!("  [echo] saw: {}  kind={}", event.id, event.kind);
        Ok(None)
    }
}

/// An example plugin that counts events using memory storage.
///
/// ```
/// use pantheon_plugin::{PantheonPlugin, PluginContext, CounterPlugin};
/// use pantheon_kernel::error::Result;
///
/// let mut p = CounterPlugin::new();
/// assert_eq!(p.name(), "counter");
/// ```
pub struct CounterPlugin {
    ctx: Option<PluginContext>,
    key: String,
}

impl CounterPlugin {
    pub fn new() -> Self {
        Self {
            ctx: None,
            key: "counter.events".to_string(),
        }
    }

    pub fn with_key(mut self, key: &str) -> Self {
        self.key = key.to_string();
        self
    }
}

impl PantheonPlugin for CounterPlugin {
    fn name(&self) -> &'static str {
        "counter"
    }

    fn description(&self) -> &'static str {
        "Demonstrates memory-backed state by counting events."
    }

    fn init(&mut self, ctx: PluginContext) -> Result<()> {
        self.ctx = Some(ctx);
        Ok(())
    }

    fn on_event(&self, _event: &Event) -> Result<Option<Event>> {
        if let Some(ref ctx) = self.ctx {
            let mut mem = ctx.memory.lock().unwrap();
            let count = mem
                .get(&self.key)
                .and_then(|e| {
                    let s = String::from_utf8_lossy(&e.value).to_string();
                    s.parse::<u64>().ok()
                })
                .unwrap_or(0);
            let new_count = count + 1;
            mem.put(
                &self.key,
                new_count.to_string().into_bytes(),
                CellId::random(),
            )?;
            ctx.metrics
                .lock()
                .unwrap()
                .increment("counter.events.seen");
        }
        Ok(None)
    }
}

/// An example plugin that cancels tasks matching a condition.
///
/// ```
/// use pantheon_plugin::{PantheonPlugin, PluginContext, FilterPlugin};
/// use pantheon_kernel::error::Result;
///
/// let p = FilterPlugin::new("reject-me");
/// assert_eq!(p.name(), "filter");
/// ```
pub struct FilterPlugin {
    reject_substring: String,
}

impl FilterPlugin {
    pub fn new(reject_substring: &str) -> Self {
        Self {
            reject_substring: reject_substring.to_string(),
        }
    }
}

impl PantheonPlugin for FilterPlugin {
    fn name(&self) -> &'static str {
        "filter"
    }

    fn description(&self) -> &'static str {
        "Cancels tasks whose payload contains a configured substring."
    }

    fn init(&mut self, _ctx: PluginContext) -> Result<()> {
        Ok(())
    }

    fn on_task(&self, task: &ScheduledTask) -> Result<TaskAction> {
        let payload = String::from_utf8_lossy(task.payload().as_bytes());
        if payload.contains(&self.reject_substring) {
            Ok(TaskAction::Cancel(format!(
                "FilterPlugin rejected: contains '{}'",
                self.reject_substring
            )))
        } else {
            Ok(TaskAction::Pass)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pantheon_kernel::event::EventPayload;
    use std::sync::Arc;
    use std::sync::Mutex;
    use pantheon_event_bus::bus::EventBus;
    use pantheon_scheduler::Scheduler;
    use pantheon_capabilities::manager::CapabilityManager;
    use pantheon_metrics::MetricsCollector;

    fn dummy_ctx() -> PluginContext {
        PluginContext {
            event_bus: Arc::new(Mutex::new(EventBus::new())),
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
            capabilities: Arc::new(Mutex::new(CapabilityManager::new())),
            metrics: Arc::new(Mutex::new(MetricsCollector::new())),
            memory: Arc::new(Mutex::new(MemoryManager::new())),
        }
    }

    #[test]
    fn test_echo_plugin() {
        let mut p = EchoPlugin::new();
        p.init(dummy_ctx()).unwrap();
        assert_eq!(p.name(), "echo");
    }

    #[test]
    fn test_counter_plugin() {
        let ctx = Arc::new(dummy_ctx());
        let mut p = CounterPlugin::new();
        p.init(ctx.as_ref().clone()).unwrap();

        let cell = CellId::random();
        let event = Event::new(
            pantheon_kernel::event_id::EventId::new(
                cell,
                pantheon_kernel::types::LamportTimestamp::new(1),
                pantheon_kernel::types::SequenceNumber::new(0),
            ),
            pantheon_kernel::event::EventKind::Notification,
            pantheon_kernel::protocol::ProtocolRef::new("test.v1"),
            cell,
            "test.cap",
            EventPayload::new(vec![], "text/plain"),
        );

        // Fire two events
        p.on_event(&event).unwrap();
        p.on_event(&event).unwrap();

        // Check counter in memory
        let mem = ctx.memory.lock().unwrap();
        let entry = mem.get("counter.events").unwrap();
        let val: u64 = String::from_utf8_lossy(&entry.value).parse().unwrap();
        assert_eq!(val, 2);
    }

    #[test]
    fn test_filter_plugin_approves() {
        let p = FilterPlugin::new("bad");
        let cell = CellId::random();
        let task = ScheduledTask::new(
            cell,
            EventPayload::new(b"good payload".to_vec(), "text/plain"),
            pantheon_scheduler::task::TaskPriority::Normal,
        );
        assert_eq!(p.on_task(&task).unwrap(), TaskAction::Pass);
    }

    #[test]
    fn test_filter_plugin_rejects() {
        let p = FilterPlugin::new("bad");
        let cell = CellId::random();
        let task = ScheduledTask::new(
            cell,
            EventPayload::new(b"this is bad".to_vec(), "text/plain"),
            pantheon_scheduler::task::TaskPriority::Normal,
        );
        assert_eq!(
            p.on_task(&task).unwrap(),
            TaskAction::Cancel("FilterPlugin rejected: contains 'bad'".into())
        );
    }
}
