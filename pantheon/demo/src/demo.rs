use std::time::Instant;
use pantheon_kernel::capability::{Action, CapabilityConstraints};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::Result;
use pantheon_kernel::event::EventPayload;
use pantheon_capabilities::manager::CapabilityManager;
use pantheon_event_bus::bus::EventBus;
use pantheon_scheduler::Scheduler;
use pantheon_scheduler::task::TaskPriority;
use pantheon_metrics::MetricsCollector;
use crate::runtime::RuntimeManager;
use crate::memory::DemoMemory;
use crate::timeline::TimelineDisplay;

pub struct DemoResult {
    pub timeline: TimelineDisplay,
    pub cells_created: usize,
    pub events_sent: u64,
    pub tasks_scheduled: usize,
    pub tasks_executed: usize,
    pub capabilities_issued: usize,
    pub duration: std::time::Duration,
    pub metrics_display: String,
}

pub fn run_demo() -> Result<DemoResult> {
    let start = Instant::now();
    let runtime = RuntimeManager::new();
    let memory = DemoMemory::new();
    let mut capabilities = CapabilityManager::new();
    let event_bus = EventBus::new();
    let scheduler = Scheduler::new();
    let metrics = MetricsCollector::new();

    memory.record("Demo Started", "Pantheon Autonomous Engineering OS");

    metrics.set_gauge("cells.alive", 0);

    // Step 1: Create Cell A
    let cell_a: CellId = runtime.create_cell("Orchestrator");
    metrics.set_gauge("cells.alive", 1);
    let cell_a_name = format!("Orchestrator ({})", &cell_a.to_string()[..8]);
    memory.record(
        "Cell A Created",
        format!("ID: {}, Role: Orchestrator", cell_a),
    );

    // Step 2: Create Cell B
    let cell_b: CellId = runtime.create_cell("Worker");
    metrics.set_gauge("cells.alive", 2);
    let cell_b_name = format!("Worker ({})", &cell_b.to_string()[..8]);
    memory.record(
        "Cell B Created",
        format!("ID: {}, Role: Worker", cell_b),
    );

    // Step 3: Issue Capability from orchestrator to worker
    metrics.increment("capabilities.issued");
    let cap_id = capabilities.issue(
        cell_a,
        cell_b,
        "pantheon://events/*",
        vec![Action::Read, Action::Write],
        CapabilityConstraints::unlimited(),
    )?;
    memory.record(
        "Capability Issued",
        format!("{} → {}: Read/Write on pantheon://events/*", cell_a_name, cell_b_name),
    );

    // Step 4: Verify capability works
    metrics.increment("capability.checks");
    let verify_result = capabilities.verify(&cap_id, &cell_b, &Action::Read, "pantheon://events/test");
    memory.record(
        "Capability Verified",
        format!("Result: {}", if verify_result.is_ok() { "PASS" } else { "FAIL" }),
    );

    // Step 5: Send Event from A to B
    metrics.increment("events.sent");
    let payload = EventPayload::new(
        br#"{"action": "process", "data": "hello-world"}"#.to_vec(),
        "application/json",
    );
    let event_id = event_bus.publish(cell_a, cap_id.clone(), payload)?;
    memory.record(
        "Event Sent",
        format!("{} → Event {} to capability {}", cell_a_name, event_id, cap_id),
    );

    // Step 6: Schedule a task via scheduler
    metrics.increment("tasks.scheduled");
    let task_payload = EventPayload::new(
        br#"{"command": "process-event", "event_id": "demo-1"}"#.to_vec(),
        "application/json",
    );
    let task_id = scheduler.schedule_event_driven(cell_a, task_payload, TaskPriority::High)?;
    memory.record(
        "Task Scheduled",
        format!("{} via Scheduler (priority: High)", task_id),
    );

    // Step 7: Runtime executes the task
    metrics.increment("tasks.executed");
    let task = scheduler.next_task().expect("Task should be available");
    let exec_result = runtime.execute_task(&cell_b, "process-event")?;
    scheduler.complete_task(&task.id())?;
    memory.record(
        "Task Executed",
        format!("{} → Cell B: {}", task.id(), exec_result),
    );

    // Step 8: Log and memory updated
    let log = event_bus.local_log();
    let log_len = log.lock().unwrap().len();
    memory.record(
        "Event Log Updated",
        format!("Log size: {} events stored", log_len),
    );

    let cells_alive = runtime.cell_count();
    let pending_tasks = scheduler.pending_count();
    let completed_tasks = scheduler.completed_count();
    let events_count = log_len;

    memory.record(
        "Memory Updated",
        format!("Cells: {}, Tasks completed: {}, Events stored: {}", cells_alive, completed_tasks, events_count),
    );

    // Step 9: Print final timeline
    let duration = start.elapsed();
    metrics.set_gauge("events.stored", events_count as i64);
    metrics.record_duration("demo.total", duration);

    let metrics_display = format!("{}", metrics.display());

    memory.record(
        "Demo Complete",
        format!("Total duration: {:?}", duration),
    );

    let entries = memory.timeline();
    let timeline = TimelineDisplay::new(entries);

    Ok(DemoResult {
        timeline,
        cells_created: cells_alive,
        events_sent: events_count as u64,
        tasks_scheduled: (pending_tasks + completed_tasks as usize) as usize,
        tasks_executed: completed_tasks as usize,
        capabilities_issued: 1,
        duration,
        metrics_display,
    })
}
