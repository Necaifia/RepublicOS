use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub tick: u64,
    pub kind: String,
    pub source: String,
    pub detail: String,
}

impl LogEntry {
    pub fn display(&self) -> String {
        format!(
            "  {:>6}  {:<12}  {:<10}  {}",
            self.tick, self.timestamp, self.kind, self.detail
        )
    }
}

pub struct ReplayLog {
    pub entries: Vec<LogEntry>,
}

#[allow(dead_code)]
impl ReplayLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn record(&mut self, kind: &str, source: &str, detail: &str) {
        let tick = self.entries.len() as u64;
        self.entries.push(LogEntry {
            timestamp: chrono_now(),
            tick,
            kind: kind.to_string(),
            source: source.to_string(),
            detail: detail.to_string(),
        });
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let entries: Vec<LogEntry> = serde_json::from_str(&data)?;
        Ok(Self { entries })
    }

    pub fn replay(&self, frame_delay: Duration) {
        println!();
        println!("  ╔══════════════════════════════════════════╗");
        println!("  ║         Pantheon Event Replay          ║");
        println!("  ╚══════════════════════════════════════════╝");
        println!();
        println!("  {} entries loaded", self.entries.len());
        println!();
        println!("  {:>6}  {:<12}  {:<10}  {}", "Tick", "Timestamp", "Kind", "Detail");
        println!("  {}", "─".repeat(70));
        println!();

        for entry in &self.entries {
            println!("{}", entry.display());
            if !frame_delay.is_zero() {
                std::thread::sleep(frame_delay);
            }
        }

        println!();
        println!("  Replay complete — {} events", self.entries.len());
        println!();
    }

    pub fn frame_at(&self, tick: usize) -> Option<&LogEntry> {
        self.entries.get(tick)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[allow(dead_code)]
fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let secs = now.as_secs();
    let ms = now.as_millis() % 1000;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

pub fn run_replay(path: &str) -> io::Result<()> {
    let log = ReplayLog::load(Path::new(path))?;
    log.replay(Duration::from_millis(100));
    Ok(())
}

pub fn run_replay_interactive(path: &str) -> io::Result<()> {
    let log = ReplayLog::load(Path::new(path))?;
    let total = log.len();

    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║      Pantheon Event Replay Interactive  ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
    println!("  {} events loaded", total);
    println!("  Press Enter for next frame, 'q' to quit");
    println!();

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    for tick in 0..total {
        print!("\r\x1B[K  Frame {}/{}", tick + 1, total);
        io::stdout().flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input == "q" {
            println!("\n  Replay stopped at frame {}/{}", tick, total);
            break;
        }

        if let Some(entry) = log.frame_at(tick) {
            print!("\x1B[2A\x1B[J");
            println!("  {:>6}  {:<12}  {:<10}  {:<30}", "Tick", "Timestamp", "Kind", "Detail");
            println!("  {}", "─".repeat(70));
            println!("{}", entry.display());
            println!("  {}", "─".repeat(70));
        }

        // Move cursor back for next prompt
        println!();
    }

    println!();
    println!("  Replay complete");
    println!();

    Ok(())
}
