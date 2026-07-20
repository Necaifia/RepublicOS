use std::time::Instant;
use pantheon_kernel::capability::{Action, CapabilityConstraints};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::event::{EventKind, EventPayload};
use pantheon_capabilities::manager::CapabilityManager;
use pantheon_event_bus::bus::EventBus;
use pantheon_scheduler::Scheduler;
use pantheon_scheduler::task::{ScheduledTask, TaskPriority};

fn bench<F>(name: &str, iterations: u64, f: F)
where
    F: Fn(),
{
    // warmup
    for _ in 0..10 { f(); }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() as f64 / iterations as f64;
    let throughput = (iterations as f64 / elapsed.as_secs_f64()) as u64;
    println!("  {:40} {:>8.1} ns/op  {:>10} ops/s  ({:>4} iterations in {:?})",
        name, avg_ns, throughput, iterations, elapsed);
}

fn bench_cell_creation() {
    println!("\n── Cell Creation ──");
    bench("CellId::random() x100", 100, || {
        for _ in 0..100 { std::hint::black_box(CellId::random()); }
    });
    bench("CellId::random() x1000", 10, || {
        for _ in 0..1000 { std::hint::black_box(CellId::random()); }
    });
    bench("CellId::random() x10000", 5, || {
        for _ in 0..10000 { std::hint::black_box(CellId::random()); }
    });
}

fn bench_capability() {
    println!("\n── Capability System ──");
    let issuer = CellId::random();
    let subject = CellId::random();

    bench("issue x 100", 10, || {
        let mut mgr = CapabilityManager::new();
        for _ in 0..100 {
            let _ = mgr.issue(
                issuer, subject, "pantheon://bench/*",
                vec![Action::Read, Action::Write],
                CapabilityConstraints::unlimited(),
            );
        }
    });

    {
        let mut mgr = CapabilityManager::new();
        let id = mgr.issue(issuer, subject, "pantheon://bench/*", vec![Action::Read], CapabilityConstraints::unlimited()).unwrap();
        bench("verify x 1000", 10, || {
            for _ in 0..1000 {
                let _ = mgr.verify(&id, &subject, &Action::Read, "pantheon://bench/test");
            }
        });
    }

    {
        let mut mgr = CapabilityManager::new();
        let mut id = mgr.issue(issuer, subject, "pantheon://bench/*", vec![Action::Read], CapabilityConstraints {
            delegation_depth: 20,
            ..CapabilityConstraints::unlimited()
        }).unwrap();
        let mut current = subject;
        for _ in 0..10 {
            let next = CellId::random();
            id = mgr.delegate(&id, next, None, None, None).unwrap();
            current = next;
        }
        bench("check_chain depth=10 x 100", 10, || {
            for _ in 0..100 {
                let _ = mgr.check_chain(&id, &current, &Action::Read, "pantheon://bench/test");
            }
        });
    }
}

fn bench_event_bus() {
    println!("\n── Event Bus ──");
    let source = CellId::random();

    bench("publish x 100", 10, || {
        let bus = EventBus::new();
        for _ in 0..100 {
            let _ = bus.publish(source, "bench.cap".into(), EventPayload::new(vec![0; 64], "text/plain"));
        }
    });

    bench("publish x 1000", 5, || {
        let bus = EventBus::new();
        for _ in 0..1000 {
            let _ = bus.publish(source, "bench.cap".into(), EventPayload::new(vec![0; 64], "text/plain"));
        }
    });

    {
        let bus = EventBus::new();
        for i in 0..10 {
            bus.subscribe(format!("sub-{}", i), "bench.*", vec![EventKind::Event], Box::new(move |_| Ok(())));
        }
        bench("dispatch to 10 subs x 100", 10, || {
            for _ in 0..100 {
                let _ = bus.publish(source, "bench.cap".into(), EventPayload::new(vec![0; 32], "text/plain"));
            }
        });
    }
}

fn bench_scheduler() {
    println!("\n── Scheduler ──");
    let priority_cycle = [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low, TaskPriority::Background];

    bench("schedule 100", 10, || {
        let s = Scheduler::new();
        for i in 0..100 {
            let _ = s.schedule(ScheduledTask::new(
                CellId::random(), EventPayload::new(vec![0; 32], "text/plain"), priority_cycle[i % 5],
            ));
        }
    });

    bench("schedule 1000", 5, || {
        let s = Scheduler::new();
        for i in 0..1000 {
            let _ = s.schedule(ScheduledTask::new(
                CellId::random(), EventPayload::new(vec![0; 32], "text/plain"), priority_cycle[i % 5],
            ));
        }
    });

    {
        let s = Scheduler::new();
        for i in 0..1000 {
            let _ = s.schedule(ScheduledTask::new(
                CellId::random(), EventPayload::new(vec![0; 32], "text/plain"), priority_cycle[i % 5],
            ));
        }
        bench("next_task x 1000", 5, || {
            let mut count = 0;
            while s.next_task().is_some() { count += 1; }
            std::hint::black_box(count);
        });
    }
}

fn main() {
    println!("══════════════════════════════════════════");
    println!("      Pantheon Benchmark Suite");
    println!("══════════════════════════════════════════");

    bench_cell_creation();
    bench_capability();
    bench_event_bus();
    bench_scheduler();

    println!("\n══════════════════════════════════════════");
    println!("      All benchmarks complete");
    println!("══════════════════════════════════════════");
}
