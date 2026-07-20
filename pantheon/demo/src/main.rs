use pantheon_kernel::error::Result;

mod runtime;
mod memory;
mod timeline;
mod demo;
mod simulate;
mod visualizer;
mod doctor;
mod dashboard;
mod pipeline;
mod project;
mod replay;
mod snapshot;
mod verify;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pantheon=info".into()),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

fn main() -> Result<()> {
    init_tracing();
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match command {
        "demo" => {
            tracing::info!("Starting Pantheon demo");
            let result = demo::run_demo()?;
            println!("{}", result.timeline);
            println!();
            println!("=== Summary ===");
            println!("  Cells created:  {}", result.cells_created);
            println!("  Events sent:    {}", result.events_sent);
            println!("  Tasks scheduled: {}", result.tasks_scheduled);
            println!("  Tasks executed:  {}", result.tasks_executed);
            println!("  Capabilities:    {}", result.capabilities_issued);
            println!("  Duration:        {:?}", result.duration);
            println!("  Status:         OK");
            println!();
            println!("{}", result.metrics_display);
            tracing::info!(cells = result.cells_created, events = result.events_sent, "Demo completed");
            Ok(())
        }
        "simulate" => {
            let cell_count: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000);
            tracing::info!(cell_count, "Starting simulation");
            let result = simulate::run_simulation(cell_count)?;
            println!();
            println!("=== Simulation Report ===");
            println!("  Cells created:      {}", result.cells_created);
            println!("  Total events:       {}", result.total_events);
            println!("  Total capabilities: {}", result.total_capabilities);
            println!("  Total tasks:        {}", result.total_tasks);
            println!("  Failures simulated: {}", result.failures_simulated);
            println!("  Partitions:         {}", result.partitions_simulated);
            println!("  Expired caps:       {}", result.expired_capabilities);
            println!("  Duration:           {:?}", result.duration);
            println!();
            println!("{}", result.metrics);
            tracing::info!(cells = result.cells_created, tasks = result.total_tasks, "Simulation completed");
            Ok(())
        }
        "visualize" => {
            tracing::info!("Starting visualizer");
            let v = visualizer::Visualizer::new(500);
            {
                let graph_arc = v.graph();
                let mut g = graph_arc.lock().unwrap();
                *g = visualizer::Visualizer::build_demo_graph();
            }
            v.start();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                {
                    let graph_arc = v.graph();
                    let mut g = graph_arc.lock().unwrap();
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    let events = rng.gen_range(1800..2600);
                    let caps = rng.gen_range(700..1000);
                    g.set_metric("Events/sec", format!("{}", events));
                    g.set_metric("Cap. Checks/sec", format!("{}", caps));
                    let alive = rng.gen_range(3..5);
                    g.set_metric("Cells Alive", format!("{}", alive));
                }
            }
        }
        "doctor" => {
            let checks = doctor::run_doctor();
            println!();
            println!("  ╔══════════════════════════════════════════╗");
            println!("  ║          Pantheon System Health         ║");
            println!("  ╚══════════════════════════════════════════╝");
            println!();
            for check in &checks {
                println!("  {} {}: {}", check.status, check.name, check.detail);
            }
            println!();
            let failed = checks.iter().filter(|c| c.status == doctor::HealthStatus::Fail).count();
            if failed > 0 {
                println!("  {} check(s) FAILED", failed);
            } else {
                println!("  All {} checks passed", checks.len());
            }
            Ok(())
        }
        "inspect" => {
            let report = doctor::run_inspect();
            println!("{}", report);
            Ok(())
        }
        "dashboard" => {
            let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8080);
            tracing::info!(port, "Starting dashboard");
            println!("  Dashboard: http://localhost:{}", port);
            println!("  Live event timeline + system metrics");
            println!("  Press Ctrl+C to stop");
            println!();
            dashboard::run_dashboard(port).map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            Ok(())
        }
        "init" => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("my-project");
            tracing::info!(name, "Initializing project");
            println!();
            println!("  ╔══════════════════════════════════════════╗");
            println!("  ║        Pantheon Project Init           ║");
            println!("  ╚══════════════════════════════════════════╝");
            println!();
            project::init_project(name)
                .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            Ok(())
        }
        "run" => {
            let with_human = args.iter().any(|a| a == "--with-human");
            tracing::info!(with_human, "Starting pipeline");
            if with_human {
                use std::sync::{Arc, Mutex};
                use pantheon_event_bus::bus::EventBus;
                use pantheon_scheduler::Scheduler;
                use pantheon_capabilities::manager::CapabilityManager;
                use pantheon_metrics::MetricsCollector;
                use pantheon_plugin::{PantheonPlugin, PluginContext};
                use pantheon_plugin::local_human::LocalHumanPlugin;

                let ctx = PluginContext {
                    event_bus: Arc::new(Mutex::new(EventBus::new())),
                    scheduler: Arc::new(Mutex::new(Scheduler::new())),
                    capabilities: Arc::new(Mutex::new(CapabilityManager::new())),
                    metrics: Arc::new(Mutex::new(MetricsCollector::new())),
                    memory: Arc::new(Mutex::new(pantheon_memory::MemoryManager::new())),
                };

                let mut plugin = LocalHumanPlugin::new();
                plugin.init(ctx).map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;

                let pipeline = pipeline::Pipeline::todo();
                pipeline.run_with_plugin(&plugin);
            } else {
                let pipeline = pipeline::Pipeline::todo();
                pipeline.run();
            }
            Ok(())
        }
        "replay" => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or("events.log");
            let interactive = args.iter().any(|a| a == "--interactive" || a == "-i");
            tracing::info!(path, interactive, "Starting replay");
            if interactive {
                replay::run_replay_interactive(path)
                    .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            } else {
                replay::run_replay(path)
                    .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            }
            Ok(())
        }
        "snapshot" => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or("snapshot.ptn");
            tracing::info!(path, "Taking snapshot");
            snapshot::run_snapshot(path)
                .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            Ok(())
        }
        "restore" => {
            let default_path = "snapshot.ptn".to_string();
            let path = args.get(2).unwrap_or(&default_path);
            tracing::info!(path, "Restoring snapshot");
            snapshot::run_restore(path)
                .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            Ok(())
        }
        "verify" => {
            let path = args.get(2).map(|s| s.as_str()).unwrap_or("events.log");
            tracing::info!(path, "Verifying system determinism");
            verify::run_verify(path)
                .map_err(|e| pantheon_kernel::error::Error::Custom(e.to_string()))?;
            Ok(())
        }
        _ => {
            eprintln!("Usage: pantheon <command>");
            eprintln!();
            eprintln!("Pipeline:");
            eprintln!("  init [name]       Create a new pipeline project (default: my-project)");
            eprintln!("  run               Run the Todo App pipeline demo");
            eprintln!("    --with-human    Run with human-as-a-cell plugin (interactive approval)");
            eprintln!();
            eprintln!("System:");
            eprintln!("  demo              Run end-to-end system demonstration");
            eprintln!("  simulate [N]      Run simulation with N cells (default: 1000)");
            eprintln!("  visualize         Launch live ASCII visualizer");
            eprintln!();
            eprintln!("Debug:");
            eprintln!("  doctor            Run system health checks");
            eprintln!("  inspect           Show live system inspection");
            eprintln!("  dashboard [port]  Launch web dashboard (default: 8080)");
            eprintln!();
            eprintln!("Tools:");
            eprintln!("  replay [file]     Replay event log (--interactive for frame-by-frame)");
            eprintln!("  snapshot [file]   Save system state snapshot (default: snapshot.ptn)");
            eprintln!("  restore [file]    Restore system from snapshot");
            eprintln!("  verify [file]     Verify determinism of event log");
            std::process::exit(1);
        }
    }
}
