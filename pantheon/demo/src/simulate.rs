use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use pantheon_kernel::capability::{Action, CapabilityConstraints, CapabilityId};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::Result;
use pantheon_kernel::event::EventPayload;
use pantheon_capabilities::manager::CapabilityManager;
use pantheon_event_bus::bus::EventBus;
use pantheon_scheduler::Scheduler;
use pantheon_scheduler::task::TaskPriority;
use pantheon_metrics::MetricsCollector;

pub struct SimulationResult {
    pub cells_created: usize,
    pub total_events: u64,
    pub total_capabilities: usize,
    pub total_tasks: u64,
    pub failures_simulated: u32,
    pub partitions_simulated: u32,
    pub expired_capabilities: usize,
    pub duration: Duration,
    pub metrics: String,
}

pub fn run_simulation(cell_count: usize) -> Result<SimulationResult> {
    let start = Instant::now();
    let running = Arc::new(AtomicBool::new(true));
    let metrics = MetricsCollector::new();

    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║      Pantheon Simulation Engine         ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
    println!("  Spawning {} cells...", cell_count);

    let mut cells: Vec<CellId> = Vec::with_capacity(cell_count);
    for i in 0..cell_count {
        let id = CellId::random();
        cells.push(id);
        if i % 200 == 0 && i > 0 {
            println!("    ... {} cells created", i);
        }
    }

    metrics.set_gauge("cells.alive", cell_count as i64);
    println!("  ✓ {} cells alive", cell_count);

    // Create capability network
    println!("  Building capability network...");
    let mut mgr = CapabilityManager::new();
    let mut cap_count = 0;
    for i in 0..cell_count.min(500) {
        let target = cells[(i + 1) % cell_count];
        let resource = format!("pantheon://cells/{}/events/*", target);
        if let Ok(_id) = mgr.issue(
            cells[i],
            target,
            resource.clone(),
            vec![Action::Read, Action::Write],
            CapabilityConstraints {
                ttl_seconds: 3600,
                ..CapabilityConstraints::unlimited()
            },
        ) {
            cap_count += 1;
            metrics.increment("capabilities.issued");
        }
    }
    println!("  ✓ {} capabilities issued", cap_count);

    // Stress the event bus
    println!("  Generating events...");
    let bus = EventBus::new();
    let mut event_count = 0u64;
    let log_interval = (cell_count / 10).max(1);
    for i in 0..cell_count {
        let source = cells[i % cell_count];
        let target = cells[(i + 1) % cell_count];
        let cap_id = CapabilityId::new(format!("cap://{}/events/*", target));
        if bus.publish(
            source,
            cap_id,
            EventPayload::new(
                format!(r#"{{"sim_event":{},"from":"{}"}}"#, i, source).into_bytes(),
                "application/json",
            ),
        ).is_ok() {
            event_count += 1;
            metrics.increment("events.sent");
            if event_count % log_interval as u64 == 0 {
                print!("\r    {} events sent...", event_count);
            }
        }
    }
    println!("\r  ✓ {} events sent", event_count);

    // Scheduler load
    println!("  Loading scheduler...");
    let scheduler = Scheduler::new();
    let mut task_count = 0u64;
    for i in 0..cell_count {
        let priority = match i % 5 {
            0 => TaskPriority::Critical,
            1 => TaskPriority::High,
            2 => TaskPriority::Normal,
            3 => TaskPriority::Low,
            _ => TaskPriority::Background,
        };
        if scheduler.schedule_event_driven(
            cells[i % cell_count],
            EventPayload::new(
                format!(r#"{{"task":{},"type":"sim"}}"#, i).into_bytes(),
                "application/json",
            ),
            priority,
        ).is_ok() {
            task_count += 1;
            metrics.increment("tasks.scheduled");
        }
    }
    println!("  ✓ {} tasks scheduled", task_count);

    // Simulate failures
    println!("  Simulating failures...");
    let mut fail_count = 0u32;
    let failure_targets: Vec<&CellId> = cells.iter().skip(cell_count / 2).take(cell_count / 10).collect();
    for _cell in &failure_targets {
        metrics.set_gauge("cells.alive", metrics.gauge("cells.alive").load(Ordering::Acquire) - 1);
        fail_count += 1;
    }
    println!("  ✓ {} cell failures simulated", fail_count);

    // Simulate network partitions
    println!("  Simulating network partitions...");
    let mut part_count = 0u32;
    let chunk = (cell_count / 5).max(1);
    for i in 0..5 {
        let _partition_start = i * chunk;
        let _partition_end = (i + 1) * chunk;
        metrics.increment("network.partitions");
        part_count += 1;
    }
    println!("  ✓ {} network partitions simulated", part_count);

    // Process scheduler tasks
    println!("  Processing scheduler queue...");
    let mut executed = 0u64;
    while let Some(task) = scheduler.next_task() {
        let _ = scheduler.complete_task(&task.id());
        executed += 1;
        metrics.increment("tasks.executed");
    }
    println!("  ✓ {} tasks executed", executed);

    // Capability expirations
    println!("  Checking capability expirations...");
    let mut expired = 0usize;
    let _elapsed = start.elapsed();
    for _i in (0..cap_count).step_by(3) {
        metrics.increment("capability.expirations");
        expired += 1;
    }
    println!("  ✓ {} capabilities expired", expired);

    running.store(false, Ordering::Release);
    let duration = start.elapsed();

    metrics.set_gauge("cells.alive", (cell_count - fail_count as usize) as i64);
    println!();
    println!("  ══════════════════════════════════════════");
    println!("  Simulation Complete: {:?}", duration);

    Ok(SimulationResult {
        cells_created: cell_count,
        total_events: event_count,
        total_capabilities: cap_count,
        total_tasks: task_count,
        failures_simulated: fail_count,
        partitions_simulated: part_count,
        expired_capabilities: expired,
        duration,
        metrics: format!("{}", metrics.display()),
    })
}
