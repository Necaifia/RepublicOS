use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub active: bool,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct SystemGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metrics: HashMap<String, String>,
}

impl SystemGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: impl Into<String>, label: impl Into<String>, x: usize, y: usize) {
        self.nodes.push(GraphNode {
            id: id.into(),
            label: label.into(),
            active: true,
            x,
            y,
        });
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) {
        self.edges.push(GraphEdge {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            active: true,
        });
    }

    pub fn set_metric(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metrics.insert(key.into(), value.into());
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str("  ╔══════════════════════════════════════════╗\n");
        output.push_str("  ║        Pantheon System Visualizer       ║\n");
        output.push_str("  ╚══════════════════════════════════════════╝\n");
        output.push('\n');

        let max_x = self.nodes.iter().map(|n| n.x).max().unwrap_or(10) + 1;
        let max_y = self.nodes.iter().map(|n| n.y).max().unwrap_or(10) + 1;

        let node_map: HashMap<&str, &GraphNode> =
            self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        for y in 0..max_y {
            for x in 0..max_x {
                let node_at = self.nodes.iter().find(|n| n.x == x && n.y == y);

                if let Some(node) = node_at {
                    if node.active {
                        output.push_str(" ● ");
                    } else {
                        output.push_str(" ○ ");
                    }
                } else {
                    let has_h_edge = self.edges.iter().any(|e| {
                        let from = node_map.get(e.from.as_str());
                        let to = node_map.get(e.to.as_str());
                        if let (Some(f), Some(t)) = (from, to) {
                            if f.y == y && f.y == t.y {
                                (f.x == x && t.x > x) || (f.x < x && t.x == x)
                                    || (x > f.x && x < t.x)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });

                    let has_v_edge = self.edges.iter().any(|e| {
                        let from = node_map.get(e.from.as_str());
                        let to = node_map.get(e.to.as_str());
                        if let (Some(f), Some(t)) = (from, to) {
                            if f.x == x && f.x == t.x {
                                (f.y == y && t.y > y) || (f.y < y && t.y == y)
                                    || (y > f.y && y < t.y)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });

                    if has_h_edge {
                        output.push_str("───");
                    } else if has_v_edge {
                        output.push_str(" │ ");
                    } else {
                        output.push_str("   ");
                    }
                }
            }
            output.push('\n');

            // Labels row
            for x in 0..max_x {
                if let Some(node) = self.nodes.iter().find(|n| n.x == x && n.y == y) {
                    let label = &node.label;
                    let padded = if label.len() > 12 {
                        format!("{:.11}.", label)
                    } else {
                        format!("{:12}", label)
                    };
                    output.push_str(&format!("{:<13}", padded));
                } else {
                    output.push_str("             ");
                }
            }
            output.push('\n');
        }

        // Metrics panel
        if !self.metrics.is_empty() {
            output.push('\n');
            output.push_str("  ─── Metrics ───\n");
            let mut keys: Vec<&String> = self.metrics.keys().collect();
            keys.sort();
            for key in keys {
                output.push_str(&format!("    {:30} {}\n", key, self.metrics[key]));
            }
        }

        output
    }
}

pub struct Visualizer {
    graph: Arc<Mutex<SystemGraph>>,
    running: Arc<AtomicBool>,
    refresh_ms: u64,
}

impl Visualizer {
    pub fn new(refresh_ms: u64) -> Self {
        Self {
            graph: Arc::new(Mutex::new(SystemGraph::new())),
            running: Arc::new(AtomicBool::new(false)),
            refresh_ms,
        }
    }

    pub fn graph(&self) -> Arc<Mutex<SystemGraph>> {
        self.graph.clone()
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::Release);
        let running = self.running.clone();
        let graph = self.graph.clone();
        let refresh = self.refresh_ms;

        std::thread::spawn(move || {
            while running.load(Ordering::Acquire) {
                let rendered = {
                    let g = graph.lock().unwrap();
                    g.render()
                };
                // Clear screen and move cursor to top
                print!("\x1B[2J\x1B[H");
                println!("{}", rendered);
                std::thread::sleep(Duration::from_millis(refresh));
            }
        });
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    pub fn build_demo_graph() -> SystemGraph {
        let mut g = SystemGraph::new();

        // 3-tier topology
        g.add_node("coordinator", "Coordinator", 2, 0);
        g.add_node("worker-1", "Worker 1", 0, 2);
        g.add_node("worker-2", "Worker 2", 4, 2);
        g.add_node("worker-3", "Worker 3", 2, 4);

        g.add_edge("coordinator", "worker-1", "events");
        g.add_edge("coordinator", "worker-2", "events");
        g.add_edge("worker-1", "worker-3", "data");
        g.add_edge("worker-2", "worker-3", "data");

        g.set_metric("Cells Alive", "4");
        g.set_metric("Events/sec", "2,200");
        g.set_metric("Cap. Checks/sec", "880");
        g.set_metric("Scheduling (avg)", "1.4ms");
        g.set_metric("Protocol Messages", "3,200");

        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut g = SystemGraph::new();
        g.add_node("a", "Node A", 0, 0);
        g.add_node("b", "Node B", 2, 0);
        g.add_edge("a", "b", "link");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn test_graph_render_contains_nodes() {
        let g = Visualizer::build_demo_graph();
        let rendered = g.render();
        assert!(rendered.contains("Coordinator"));
        assert!(rendered.contains("Worker 1"));
        assert!(rendered.contains("Worker 2"));
        assert!(rendered.contains("Worker 3"));
    }

    #[test]
    fn test_graph_render_contains_metrics() {
        let g = Visualizer::build_demo_graph();
        let rendered = g.render();
        assert!(rendered.contains("Cells Alive"));
        assert!(rendered.contains("Events/sec"));
    }

    #[test]
    fn test_visualizer_start_stop() {
        let v = Visualizer::new(100);
        v.start();
        std::thread::sleep(Duration::from_millis(50));
        v.stop();
    }

    #[test]
    fn test_graph_metrics() {
        let mut g = SystemGraph::new();
        g.set_metric("test.key", "42");
        let rendered = g.render();
        assert!(rendered.contains("test.key"));
        assert!(rendered.contains("42"));
    }

    #[test]
    fn test_inactive_node() {
        let mut g = SystemGraph::new();
        g.add_node("dead", "Dead Cell", 0, 0);
        g.nodes[0].active = false;
        let rendered = g.render();
        assert!(rendered.contains("○"));
    }
}
