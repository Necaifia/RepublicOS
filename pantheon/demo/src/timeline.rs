use crate::memory::TimelineEntry;
use std::fmt;

pub struct TimelineDisplay {
    entries: Vec<TimelineEntry>,
}

impl TimelineDisplay {
    pub fn new(entries: Vec<TimelineEntry>) -> Self {
        Self { entries }
    }
}

impl fmt::Display for TimelineDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_ns = self.entries.first().map(|e| e.timestamp_ns).unwrap_or(0);
        writeln!(f, "")?;
        writeln!(f, "  ╔══════════════════════════════════════════════════╗")?;
        writeln!(f, "  ║           Pantheon System Timeline              ║")?;
        writeln!(f, "  ╚══════════════════════════════════════════════════╝")?;
        writeln!(f, "")?;

        for entry in &self.entries {
            let t = entry.timestamp_ns.saturating_sub(start_ns);
            let t_ms = t as f64 / 1_000_000.0;
            let icon = match entry.action.as_str() {
                a if a.contains("Cell") => "□",
                a if a.contains("Capability") => "🔑",
                a if a.contains("Event") => "➤",
                a if a.contains("Schedule") => "⏱",
                a if a.contains("Execute") => "⚡",
                a if a.contains("Log") => "📝",
                a if a.contains("Memory") => "💾",
                a if a.contains("Complete") => "✓",
                _ => "·",
            };
            writeln!(
                f,
                "  {:>3}. [{:+9.3}ms] {} {}",
                entry.step, t_ms, icon, entry.action
            )?;
            writeln!(f, "       {}", entry.detail)?;
        }

        writeln!(f, "")?;
        writeln!(f, "  ─────────────────────────────────────────────")?;
        writeln!(
            f,
            "  Total steps: {}, Duration: {:.3}ms",
            self.entries.len(),
            self.entries
                .last()
                .map(|e| (e.timestamp_ns.saturating_sub(start_ns)) as f64 / 1_000_000.0)
                .unwrap_or(0.0)
        )?;
        Ok(())
    }
}
