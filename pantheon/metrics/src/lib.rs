use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct MetricsCollector {
    counters: Mutex<HashMap<String, Arc<AtomicU64>>>,
    gauges: Mutex<HashMap<String, Arc<AtomicI64>>>,
    histograms: Mutex<HashMap<String, Vec<u64>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    pub fn counter(&self, name: &str) -> Arc<AtomicU64> {
        let mut counters = self.counters.lock().unwrap();
        counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
    }

    pub fn increment(&self, name: &str) -> u64 {
        let c = self.counter(name);
        c.fetch_add(1, Ordering::Release)
    }

    pub fn add(&self, name: &str, value: u64) -> u64 {
        let c = self.counter(name);
        c.fetch_add(value, Ordering::Release)
    }

    pub fn gauge(&self, name: &str) -> Arc<AtomicI64> {
        let mut gauges = self.gauges.lock().unwrap();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicI64::new(0)))
            .clone()
    }

    pub fn set_gauge(&self, name: &str, value: i64) -> i64 {
        let g = self.gauge(name);
        g.fetch_add(0, Ordering::Acquire); // touch
        g.store(value, Ordering::Release);
        value
    }

    pub fn record_histogram(&self, name: &str, value_ns: u64) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms.entry(name.to_string()).or_default().push(value_ns);
    }

    pub fn record_duration(&self, name: &str, duration: Duration) {
        self.record_histogram(name, duration.as_nanos() as u64);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.lock().unwrap();
        let gauges = self.gauges.lock().unwrap();
        let histograms = self.histograms.lock().unwrap();

        let mut counter_map = HashMap::new();
        for (k, v) in counters.iter() {
            counter_map.insert(k.clone(), v.load(Ordering::Acquire));
        }

        let mut gauge_map = HashMap::new();
        for (k, v) in gauges.iter() {
            gauge_map.insert(k.clone(), v.load(Ordering::Acquire));
        }

        let mut histogram_snapshot = HashMap::new();
        for (k, v) in histograms.iter() {
            let mut sorted = v.clone();
            sorted.sort_unstable();
            let count = sorted.len();
            let sum: u64 = sorted.iter().sum();
            let avg = if count > 0 { sum / count as u64 } else { 0 };
            let p50 = percentile(&sorted, 50.0);
            let p95 = percentile(&sorted, 95.0);
            let p99 = percentile(&sorted, 99.0);
            histogram_snapshot.insert(
                k.clone(),
                HistogramSummary {
                    count,
                    sum,
                    avg,
                    min: sorted.first().copied().unwrap_or(0),
                    max: sorted.last().copied().unwrap_or(0),
                    p50,
                    p95,
                    p99,
                },
            );
        }

        MetricsSnapshot {
            counters: counter_map,
            gauges: gauge_map,
            histograms: histogram_snapshot,
            uptime: self.start_time.elapsed(),
        }
    }

    pub fn display(&self) -> MetricsDisplay {
        MetricsDisplay::new(self.snapshot())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HistogramSummary {
    pub count: usize,
    pub sum: u64,
    pub avg: u64,
    pub min: u64,
    pub max: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

impl HistogramSummary {
    pub fn avg_ms(&self) -> f64 {
        self.avg as f64 / 1_000_000.0
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, i64>,
    pub histograms: HashMap<String, HistogramSummary>,
    pub uptime: Duration,
}

pub struct MetricsDisplay {
    snapshot: MetricsSnapshot,
}

impl MetricsDisplay {
    pub fn new(snapshot: MetricsSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.snapshot).unwrap_or_default()
    }
}

impl std::fmt::Display for MetricsDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = &self.snapshot;
        writeln!(f, "")?;
        writeln!(f, "  ╔══════════════════════════════════════════╗")?;
        writeln!(f, "  ║           Pantheon Metrics              ║")?;
        writeln!(f, "  ╚══════════════════════════════════════════╝")?;
        writeln!(f, "")?;

        if !s.counters.is_empty() {
            writeln!(f, "  Counters:")?;
            let mut keys: Vec<&String> = s.counters.keys().collect();
            keys.sort();
            for k in keys {
                writeln!(f, "    {:40} {}", k, s.counters[k])?;
            }
            writeln!(f, "")?;
        }

        if !s.gauges.is_empty() {
            writeln!(f, "  Gauges:")?;
            let mut keys: Vec<&String> = s.gauges.keys().collect();
            keys.sort();
            for k in keys {
                writeln!(f, "    {:40} {}", k, s.gauges[k])?;
            }
            writeln!(f, "")?;
        }

        if !s.histograms.is_empty() {
            writeln!(f, "  Latency Histograms (nanoseconds):")?;
            let mut keys: Vec<&String> = s.histograms.keys().collect();
            keys.sort();
            for k in keys {
                let h = &s.histograms[k];
                writeln!(f, "    {:40}", k)?;
                writeln!(f, "      count: {:8}  avg: {:>8.3}ms  p50: {:>8.3}ms  p95: {:>8.3}ms  p99: {:>8.3}ms",
                    h.count, h.avg_ms(), h.p50 as f64 / 1_000_000.0,
                    h.p95 as f64 / 1_000_000.0, h.p99 as f64 / 1_000_000.0)?;
            }
            writeln!(f, "")?;
        }

        writeln!(f, "  Uptime: {:?}", s.uptime)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_increment_counter() {
        let m = MetricsCollector::new();
        assert_eq!(m.increment("test.counter"), 0);
        assert_eq!(m.increment("test.counter"), 1);
    }

    #[test]
    fn test_add_counter() {
        let m = MetricsCollector::new();
        m.add("test.add", 10);
        m.add("test.add", 5);
        let snap = m.snapshot();
        assert_eq!(snap.counters["test.add"], 15);
    }

    #[test]
    fn test_gauge() {
        let m = MetricsCollector::new();
        m.set_gauge("test.gauge", 42);
        let snap = m.snapshot();
        assert_eq!(snap.gauges["test.gauge"], 42);
    }

    #[test]
    fn test_histogram() {
        let m = MetricsCollector::new();
        m.record_histogram("test.latency", 1000);
        m.record_histogram("test.latency", 2000);
        m.record_histogram("test.latency", 3000);
        let snap = m.snapshot();
        let h = &snap.histograms["test.latency"];
        assert_eq!(h.count, 3);
        assert_eq!(h.min, 1000);
        assert_eq!(h.max, 3000);
        assert_eq!(h.avg, 2000);
    }

    #[test]
    fn test_record_duration() {
        let m = MetricsCollector::new();
        m.record_duration("test.op", Duration::from_millis(150));
        let snap = m.snapshot();
        let h = &snap.histograms["test.op"];
        assert_eq!(h.count, 1);
        assert!(h.avg >= 150_000_000);
    }

    #[test]
    fn test_multiple_counters() {
        let m = MetricsCollector::new();
        m.increment("a");
        m.increment("b");
        m.increment("a");
        let snap = m.snapshot();
        assert_eq!(snap.counters["a"], 2);
        assert_eq!(snap.counters["b"], 1);
    }

    #[test]
    fn test_gauge_reuse() {
        let m = MetricsCollector::new();
        let g1 = m.gauge("shared");
        let g2 = m.gauge("shared");
        g1.store(100, Ordering::Release);
        assert_eq!(g2.load(Ordering::Acquire), 100);
    }

    #[test]
    fn test_percentile_edge_cases() {
        let m = MetricsCollector::new();
        m.record_histogram("single", 999);
        let snap = m.snapshot();
        let h = &snap.histograms["single"];
        assert_eq!(h.p50, 999);
        assert_eq!(h.p95, 999);
        assert_eq!(h.p99, 999);
    }

    #[test]
    fn test_uptime() {
        let m = MetricsCollector::new();
        thread::sleep(Duration::from_millis(5));
        let snap = m.snapshot();
        assert!(snap.uptime.as_millis() >= 5);
    }

    #[test]
    fn test_display_format() {
        let m = MetricsCollector::new();
        m.increment("test.counter");
        m.set_gauge("test.gauge", 42);
        m.record_histogram("test.latency", 1_500_000);
        let display = m.display();
        let output = format!("{}", display);
        assert!(output.contains("test.counter"));
        assert!(output.contains("test.gauge"));
        assert!(output.contains("test.latency"));
    }

    #[test]
    fn test_to_json() {
        let m = MetricsCollector::new();
        m.increment("json.test");
        let json = m.display().to_json();
        assert!(json.contains("json.test"));
        assert!(json.contains("counters"));
    }
}
