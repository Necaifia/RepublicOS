use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSnapshot {
    pub id: String,
    pub role: String,
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySnapshot {
    pub id: String,
    pub issuer: String,
    pub subject: String,
    pub resource: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSnapshot {
    pub id: String,
    pub source: String,
    pub kind: String,
    pub data_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerSnapshot {
    pub pending: usize,
    pub completed: usize,
    pub cancelled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub label: String,
    pub timestamp: String,
    pub cells: Vec<CellSnapshot>,
    pub capabilities: Vec<CapabilitySnapshot>,
    pub events: Vec<EventSnapshot>,
    pub scheduler: SchedulerSnapshot,
}

impl SystemSnapshot {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            timestamp: now_str(),
            cells: Vec::new(),
            capabilities: Vec::new(),
            events: Vec::new(),
            scheduler: SchedulerSnapshot {
                pending: 0,
                completed: 0,
                cancelled: 0,
            },
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let ext = if path.extension().map(|e| e == "ptn").unwrap_or(false) {
            path.to_path_buf()
        } else {
            path.with_extension("ptn")
        };
        fs::write(&ext, json)?;
        println!("  ✓ Snapshot saved: {}", ext.display());
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let snap: Self = serde_json::from_str(&data)?;
        println!("  ✓ Snapshot loaded: {} ({} cells, {} events)", path.display(), snap.cells.len(), snap.events.len());
        Ok(snap)
    }

    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n  ══════ Snapshot: {} {}\n", self.label, "═".repeat(30)));
        out.push_str(&format!("  Timestamp: {}\n", self.timestamp));
        out.push_str(&format!("  Cells: {} ({} alive)\n", self.cells.len(), self.cells.iter().filter(|c| c.alive).count()));
        out.push_str(&format!("  Capabilities: {}\n", self.capabilities.len()));
        out.push_str(&format!("  Events: {}\n", self.events.len()));
        out.push_str(&format!("  Scheduler: {} pending, {} completed\n", self.scheduler.pending, self.scheduler.completed));
        out.push_str(&format!("  {}\n", "═".repeat(50)));
        out
    }
}

fn now_str() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);
    format!("{}", now.as_secs())
}

pub fn run_snapshot(path: &str) -> std::io::Result<()> {
    let mut snap = SystemSnapshot::new("live-system");

    // Capture some demo state
    for i in 0..3 {
        snap.cells.push(CellSnapshot {
            id: format!("cell-{}", i),
            role: if i == 0 { "Orchestrator".into() } else { "Worker".into() },
            alive: true,
        });
    }

    for i in 0..2 {
        snap.capabilities.push(CapabilitySnapshot {
            id: format!("cap-{}", i),
            issuer: format!("cell-{}", i),
            subject: format!("cell-{}", i + 1),
            resource: "pantheon://events/*".into(),
            actions: vec!["read".into(), "write".into()],
        });
    }

    snap.events.push(EventSnapshot {
        id: "evt-1".into(),
        source: "cell-0".into(),
        kind: "task.process".into(),
        data_size: 256,
    });

    snap.scheduler.pending = 3;
    snap.scheduler.completed = 12;
    snap.scheduler.cancelled = 1;

    let p = Path::new(path);
    snap.save(p)?;
    println!("{}", snap.display());
    Ok(())
}

pub fn run_restore(path: &str) -> std::io::Result<()> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("snapshot not found: {}", path)));
    }
    let snap = SystemSnapshot::load(p)?;
    println!("{}", snap.display());
    Ok(())
}
