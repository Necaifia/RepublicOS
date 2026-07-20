use std::cmp::Ordering;
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::event::EventPayload;
use crate::priority::PriorityLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task-{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Critical = 5,
    High = 4,
    Normal = 3,
    Low = 2,
    Background = 1,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Critical => "critical",
            TaskPriority::High => "high",
            TaskPriority::Normal => "normal",
            TaskPriority::Low => "low",
            TaskPriority::Background => "background",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(TaskPriority::Critical),
            "high" => Some(TaskPriority::High),
            "normal" => Some(TaskPriority::Normal),
            "low" => Some(TaskPriority::Low),
            "background" => Some(TaskPriority::Background),
            _ => None,
        }
    }
}

impl From<TaskPriority> for PriorityLevel {
    fn from(p: TaskPriority) -> Self {
        match p {
            TaskPriority::Critical => PriorityLevel::Critical,
            TaskPriority::High => PriorityLevel::High,
            TaskPriority::Normal => PriorityLevel::Normal,
            TaskPriority::Low => PriorityLevel::Low,
            TaskPriority::Background => PriorityLevel::Background,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskKind {
    OneShot,
    Recurring { interval_ms: u64 },
    EventDriven,
}

impl TaskKind {
    pub fn is_recurring(&self) -> bool {
        matches!(self, TaskKind::Recurring { .. })
    }

    pub fn interval_ms(&self) -> Option<u64> {
        match self {
            TaskKind::Recurring { interval_ms } => Some(*interval_ms),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    id: TaskId,
    kind: TaskKind,
    priority: TaskPriority,
    source: CellId,
    payload: EventPayload,
    status: TaskStatus,
    scheduled_at: u64,
    created_at_ns: u128,
}

impl ScheduledTask {
    pub fn new(
        source: impl Into<CellId>,
        payload: EventPayload,
        priority: TaskPriority,
    ) -> Self {
        Self {
            id: TaskId::new(0),
            kind: TaskKind::OneShot,
            priority,
            source: source.into(),
            payload,
            status: TaskStatus::Pending,
            scheduled_at: 0,
            created_at_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        }
    }

    pub fn event_driven(
        source: impl Into<CellId>,
        payload: EventPayload,
        priority: TaskPriority,
    ) -> Self {
        Self {
            kind: TaskKind::EventDriven,
            ..Self::new(source, payload, priority)
        }
    }

    pub fn recurring(
        source: impl Into<CellId>,
        payload: EventPayload,
        priority: TaskPriority,
        interval_ms: u64,
    ) -> Self {
        Self {
            kind: TaskKind::Recurring { interval_ms },
            ..Self::new(source, payload, priority)
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn set_id(&mut self, id: TaskId) {
        self.id = id;
    }

    pub fn kind(&self) -> &TaskKind {
        &self.kind
    }

    pub fn is_recurring(&self) -> bool {
        self.kind.is_recurring()
    }

    pub fn interval_ms(&self) -> Option<u64> {
        self.kind.interval_ms()
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn source(&self) -> &CellId {
        &self.source
    }

    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn scheduled_at(&self) -> u64 {
        self.scheduled_at
    }

    pub fn set_scheduled_at(&mut self, time: u64) {
        self.scheduled_at = time;
    }

    pub fn created_at_ns(&self) -> u128 {
        self.created_at_ns
    }
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.scheduled_at.cmp(&self.scheduled_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new(42);
        assert_eq!(format!("{}", id), "task-42");
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
        assert!(TaskPriority::Low > TaskPriority::Background);
    }

    #[test]
    fn test_task_priority_as_str() {
        assert_eq!(TaskPriority::Critical.as_str(), "critical");
        assert_eq!(TaskPriority::Background.as_str(), "background");
    }

    #[test]
    fn test_task_priority_from_str() {
        assert_eq!(TaskPriority::from_str("high"), Some(TaskPriority::High));
        assert_eq!(TaskPriority::from_str("unknown"), None);
    }

    #[test]
    fn test_scheduled_task_creation() {
        let cell = CellId::random();
        let payload = EventPayload::new(vec![1, 2, 3], "application/octet-stream");
        let task = ScheduledTask::new(cell, payload.clone(), TaskPriority::Normal);
        assert_eq!(task.status(), TaskStatus::Pending);
        assert!(!task.is_recurring());
    }

    #[test]
    fn test_recurring_task_properties() {
        let cell = CellId::random();
        let payload = EventPayload::new(vec![], "text/plain");
        let task = ScheduledTask::recurring(cell, payload, TaskPriority::Low, 5000);
        assert!(task.is_recurring());
        assert_eq!(task.interval_ms(), Some(5000));
    }

    #[test]
    fn test_event_driven_task_kind() {
        let cell = CellId::random();
        let payload = EventPayload::new(vec![], "text/plain");
        let task = ScheduledTask::event_driven(cell, payload, TaskPriority::High);
        assert_eq!(*task.kind(), TaskKind::EventDriven);
    }

    #[test]
    fn test_ord_highest_priority_first() {
        let cell = CellId::random();
        let payload = EventPayload::new(vec![], "text/plain");
        let mut low = ScheduledTask::new(cell, payload.clone(), TaskPriority::Low);
        let mut high = ScheduledTask::new(cell, payload, TaskPriority::High);
        low.set_scheduled_at(1);
        high.set_scheduled_at(2);
        assert!(high > low);
    }

    #[test]
    fn test_ord_same_priority_fifo() {
        let cell = CellId::random();
        let payload = EventPayload::new(vec![], "text/plain");
        let mut first = ScheduledTask::new(cell, payload.clone(), TaskPriority::Normal);
        let mut second = ScheduledTask::new(cell, payload, TaskPriority::Normal);
        first.set_scheduled_at(1);
        second.set_scheduled_at(2);
        assert!(first > second);
    }
}
