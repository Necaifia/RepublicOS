use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub step: usize,
    pub timestamp_ns: u128,
    pub action: String,
    pub detail: String,
}

pub struct DemoMemory {
    timeline: Mutex<VecDeque<TimelineEntry>>,
    step_counter: Mutex<usize>,
}

impl DemoMemory {
    pub fn new() -> Self {
        Self {
            timeline: Mutex::new(VecDeque::new()),
            step_counter: Mutex::new(0),
        }
    }

    pub fn record(&self, action: impl Into<String>, detail: impl Into<String>) -> usize {
        let mut counter = self.step_counter.lock().unwrap();
        *counter += 1;
        let step = *counter;
        let entry = TimelineEntry {
            step,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            action: action.into(),
            detail: detail.into(),
        };
        self.timeline.lock().unwrap().push_back(entry);
        step
    }

    pub fn timeline(&self) -> Vec<TimelineEntry> {
        self.timeline.lock().unwrap().iter().cloned().collect()
    }

}

impl Default for DemoMemory {
    fn default() -> Self {
        Self::new()
    }
}
