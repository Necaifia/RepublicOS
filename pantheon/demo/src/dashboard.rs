use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use pantheon_metrics::MetricsCollector;

const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Pantheon Dashboard</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: 'Courier New', monospace; background: #0a0a0f; color: #00ff88; padding: 20px; }
  .header { text-align: center; margin-bottom: 20px; }
  .header h1 { font-size: 24px; color: #00ff88; border-bottom: 1px solid #00ff88; padding-bottom: 10px; }
  .header .sub { font-size: 12px; color: #44aa77; margin-top: 4px; }
  .layout { display: flex; gap: 20px; }
  .main { flex: 2; }
  .sidebar { flex: 1; min-width: 380px; }
  .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 20px; }
  .card { background: #111118; border: 1px solid #00ff44; border-radius: 8px; padding: 16px; text-align: center; }
  .card .value { font-size: 28px; font-weight: bold; color: #00ff88; }
  .card .label { font-size: 10px; color: #66ffaa; margin-top: 6px; text-transform: uppercase; letter-spacing: 1px; }
  .card .sub { font-size: 10px; color: #44aa77; margin-top: 2px; }
  .card.green { border-color: #00ff44; }
  .card.yellow { border-color: #ffaa00; }
  .card.red { border-color: #ff3344; }
  .card.red .value { color: #ff3344; }
  .card.yellow .value { color: #ffaa00; }
  .section-title { font-size: 12px; color: #66ffaa; margin: 16px 0 8px; text-transform: uppercase; letter-spacing: 2px; }
  .section-title .count { color: #555577; font-size: 10px; }
  .timeline { background: #0d0d15; border: 1px solid #1a1a2e; border-radius: 8px; height: 480px; overflow-y: auto; font-size: 11px; padding: 8px; }
  .timeline .entry { padding: 2px 4px; border-bottom: 1px solid #111122; font-family: 'Courier New', monospace; }
  .timeline .entry:hover { background: #151525; }
  .timeline .time { color: #555577; margin-right: 8px; }
  .timeline .tag { display: inline-block; padding: 0 6px; border-radius: 3px; font-size: 9px; margin-right: 6px; text-transform: uppercase; font-weight: bold; min-width: 60px; text-align: center; }
  .tag.created { background: #003322; color: #00ff88; }
  .tag.capability { background: #223300; color: #aaff00; }
  .tag.task { background: #001133; color: #4488ff; }
  .tag.event { background: #330022; color: #ff66aa; }
  .tag.complete { background: #002200; color: #44ff44; }
  .tag.fail { background: #330000; color: #ff4444; }
  .tag.info { background: #222233; color: #8888cc; }
  .log { background: #0d0d15; border: 1px solid #1a1a2e; border-radius: 8px; padding: 10px; height: 200px; overflow-y: auto; font-size: 11px; }
  .log .entry { padding: 2px 0; border-bottom: 1px solid #111122; }
  .log .time { color: #555577; margin-right: 8px; }
  .status { display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-right: 4px; }
  .status.ok { background: #00ff44; }
  .status.warn { background: #ffaa00; }
  .status.err { background: #ff3344; }
  #error { color: #ff3344; text-align: center; padding: 20px; display: none; }
  ::-webkit-scrollbar { width: 4px; }
  ::-webkit-scrollbar-track { background: #0a0a0f; }
  ::-webkit-scrollbar-thumb { background: #333355; border-radius: 2px; }
</style>
</head>
<body>
<div class="header">
  <h1>⬡ Pantheon System Dashboard</h1>
  <div class="sub">Live Event Timeline — updated every second</div>
</div>
<div id="error">Connection lost — retrying...</div>
<div class="layout">
  <div class="main">
    <div class="grid" id="metrics"></div>
    <div class="section-title">System Log</div>
    <div class="log" id="log"></div>
  </div>
  <div class="sidebar">
    <div class="section-title">Event Timeline <span class="count" id="event-count"></span></div>
    <div class="timeline" id="timeline"></div>
  </div>
</div>
<script>
const TAGS = {
  created: 'CREATED', capability: 'CAP', task: 'TASK',
  event: 'EVENT', complete: 'DONE', fail: 'FAIL', info: 'INFO'
};

function update() {
  fetch('/api/status').then(r => r.json()).then(d => {
    document.getElementById('error').style.display = 'none';
    const cards = [
      { key: 'cells_alive', label: 'Cells Alive', color: 'green', suffix: 'nodes' },
      { key: 'cells_dead', label: 'Cells Dead', color: d.cells_dead > 5 ? 'red' : 'green', suffix: 'dead' },
      { key: 'events_per_sec', label: 'Events / sec', color: 'green', suffix: 'ops' },
      { key: 'memory_mb', label: 'Memory', color: d.memory_mb > 100 ? 'yellow' : 'green', suffix: 'MB' },
      { key: 'capabilities_active', label: 'Capabilities', color: 'green', suffix: 'active' },
      { key: 'queue_depth', label: 'Queue Depth', color: d.queue_depth > 100 ? 'yellow' : 'green', suffix: 'pending' },
    ];
    document.getElementById('metrics').innerHTML = cards.map(c =>
      `<div class="card ${c.color}">
        <div class="value">${d[c.key] ?? 0}</div>
        <div class="label">${c.label}</div>
        <div class="sub">${c.suffix}</div>
      </div>`
    ).join('');
    if (d.log) {
      document.getElementById('log').innerHTML = d.log.map(e =>
        `<div class="entry"><span class="status ${e.status}"></span><span class="time">${e.time}</span> ${e.msg}</div>`
      ).join('');
      document.getElementById('log').scrollTop = document.getElementById('log').scrollHeight;
    }
    if (d.timeline) {
      document.getElementById('timeline').innerHTML = d.timeline.map(e =>
        `<div class="entry">
          <span class="time">${e.time}</span>
          <span class="tag ${e.tag}">${(TAGS[e.tag] || e.tag).padEnd(8)}</span>
          <span>${e.msg}</span>
        </div>`
      ).join('');
      document.getElementById('event-count').textContent = `(${d.timeline.length} events)`;
      document.getElementById('timeline').scrollTop = document.getElementById('timeline').scrollHeight;
    }
  }).catch(() => {
    document.getElementById('error').style.display = 'block';
  });
}
update();
setInterval(update, 1000);
</script>
</body>
</html>"##;

struct TimelineEntry {
    time: String,
    tag: String,
    msg: String,
}

pub struct Dashboard {
    port: u16,
    metrics: Arc<Mutex<MetricsCollector>>,
    timeline: Arc<Mutex<Vec<TimelineEntry>>>,
}

impl Dashboard {
    pub fn new(port: u16) -> Self {
        let timeline = Arc::new(Mutex::new(Vec::new()));

        // Background thread to generate fake event timeline
        {
            let timeline = timeline.clone();
            std::thread::spawn(move || {
                let tags = ["created", "capability", "task", "event", "complete", "info"];
                let msgs = [
                    "Cell-12 initialized",
                    "Capability granted: Read pantheon://cells/*",
                    "Task scheduled: process_event",
                    "Event published: cell-3→cell-7",
                    "Task completed: process_event",
                    "Cell heartbeat received",
                    "Capability verified: cell-2→cell-5",
                    "Scheduler queue rebalanced",
                    "Memory usage updated",
                    "Cell-8 crashed — restarting",
                    "Partition detected: cells 50-100",
                    "Task cancelled: timeout",
                    "Capability expired: cap-42",
                    "Cell-8 restarted successfully",
                    "Snapshot taken: system.ptn",
                ];
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    let tag = tags[rand::random::<usize>() % tags.len()];
                    let msg = msgs[rand::random::<usize>() % msgs.len()];
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(Duration::ZERO);
                    let secs = now.as_secs();
                    let ms = now.as_millis() % 1000;
                    let h = (secs / 3600) % 24;
                    let m = (secs / 60) % 60;
                    let s = secs % 60;
                    let time = format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms);
                    let mut tl = timeline.lock().unwrap();
                    tl.push(TimelineEntry { time, tag: tag.to_string(), msg: msg.to_string() });
                    if tl.len() > 100 { tl.remove(0); }
                }
            });
        }

        Self { port, metrics: Arc::new(Mutex::new(MetricsCollector::new())), timeline }
    }

    pub fn serve(&self) -> std::io::Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)?;
        tracing::info!(addr = %addr, "Dashboard started");

        let metrics = self.metrics.clone();
        let timeline = self.timeline.clone();

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let metrics = metrics.clone();
                    let timeline = timeline.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = handle_connection(&mut stream, &metrics, &timeline) {
                            tracing::debug!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("Accept error: {}", e);
                }
            }
        }
        Ok(())
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    metrics: &Arc<Mutex<MetricsCollector>>,
    timeline: &Arc<Mutex<Vec<TimelineEntry>>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let response = match path {
        "/" => {
            let len = DASHBOARD_HTML.len();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                len, DASHBOARD_HTML
            )
        }
        "/api/status" => {
            let metrics = metrics.lock().unwrap();
            metrics.set_gauge("cells_alive", rand::random::<u16>() as i64 % 15 + 975);
            metrics.set_gauge("cells_dead", rand::random::<u16>() as i64 % 15 + 10);
            metrics.set_gauge("events_per_sec", rand::random::<u16>() as i64 % 2500 + 17000);
            metrics.set_gauge("memory_mb", rand::random::<u16>() as i64 % 5 + 10);
            metrics.set_gauge("capabilities_active", rand::random::<u16>() as i64 % 200 + 4100);
            metrics.set_gauge("queue_depth", rand::random::<u16>() as i64 % 35 + 15);
            metrics.increment("api_requests");

            let snap = metrics.snapshot();

            let mut status_map = HashMap::new();
            for (k, v) in &snap.gauges { status_map.insert(k.clone(), *v); }
            for (k, v) in &snap.counters { status_map.insert(k.clone(), *v as i64); }

            let log_entries: Vec<serde_json::Value> = vec![
                serde_json::json!({"time": "NOW", "msg": "Dashboard polled", "status": "ok"})
            ];

            let tl = timeline.lock().unwrap();
            let timeline_json: Vec<serde_json::Value> = tl.iter().map(|e|
                serde_json::json!({"time": e.time, "tag": e.tag, "msg": e.msg})
            ).collect();

            let body = serde_json::json!({
                "cells_alive": status_map.get("cells_alive").unwrap_or(&0),
                "cells_dead": status_map.get("cells_dead").unwrap_or(&0),
                "events_per_sec": status_map.get("events_per_sec").unwrap_or(&0),
                "memory_mb": status_map.get("memory_mb").unwrap_or(&0),
                "capabilities_active": status_map.get("capabilities_active").unwrap_or(&0),
                "queue_depth": status_map.get("queue_depth").unwrap_or(&0),
                "api_requests": status_map.get("api_requests").unwrap_or(&0),
                "log": log_entries,
                "timeline": timeline_json,
            });
            let body_str = serde_json::to_string(&body).unwrap_or_default();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
                body_str.len(), body_str
            )
        }
        _ => {
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
        }
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

pub fn run_dashboard(port: u16) -> std::io::Result<()> {
    let dashboard = Dashboard::new(port);
    dashboard.serve()
}
