use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, BinaryHeap};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::{Error, Result};
use pantheon_kernel::event::EventPayload;
use crate::task::{ScheduledTask, TaskId, TaskPriority, TaskStatus};
use crate::priority::SchedulingPolicy;

pub struct Scheduler {
    task_queue: Mutex<BinaryHeap<ScheduledTask>>,
    active_tasks: Mutex<HashMap<TaskId, ScheduledTask>>,
    completed_tasks: Mutex<HashMap<TaskId, ScheduledTask>>,
    failed_tasks: Mutex<HashMap<TaskId, String>>,
    next_task_id: AtomicU64,
    policy: SchedulingPolicy,
    time: AtomicU64,
    metrics: Arc<SchedulerMetrics>,
}

#[derive(Debug)]
pub struct SchedulerMetrics {
    pub tasks_scheduled: AtomicU64,
    pub tasks_completed: AtomicU64,
    pub tasks_failed: AtomicU64,
    pub tasks_cancelled: AtomicU64,
    pub total_scheduling_time_ns: AtomicU64,
}

impl SchedulerMetrics {
    fn new() -> Self {
        Self {
            tasks_scheduled: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            tasks_cancelled: AtomicU64::new(0),
            total_scheduling_time_ns: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            tasks_scheduled: self.tasks_scheduled.load(AtomicOrdering::Acquire),
            tasks_completed: self.tasks_completed.load(AtomicOrdering::Acquire),
            tasks_failed: self.tasks_failed.load(AtomicOrdering::Acquire),
            tasks_cancelled: self.tasks_cancelled.load(AtomicOrdering::Acquire),
            total_scheduling_time_ns: self.total_scheduling_time_ns.load(AtomicOrdering::Acquire),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_cancelled: u64,
    pub total_scheduling_time_ns: u64,
}

impl MetricsSnapshot {
    pub fn avg_scheduling_time_ns(&self) -> f64 {
        if self.tasks_scheduled == 0 {
            0.0
        } else {
            self.total_scheduling_time_ns as f64 / self.tasks_scheduled as f64
        }
    }

    pub fn completion_rate(&self) -> f64 {
        let total = self.tasks_scheduled;
        if total == 0 {
            1.0
        } else {
            self.tasks_completed as f64 / total as f64
        }
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Mutex::new(BinaryHeap::new()),
            active_tasks: Mutex::new(HashMap::new()),
            completed_tasks: Mutex::new(HashMap::new()),
            failed_tasks: Mutex::new(HashMap::new()),
            next_task_id: AtomicU64::new(1),
            policy: SchedulingPolicy::Priority,
            time: AtomicU64::new(0),
            metrics: Arc::new(SchedulerMetrics::new()),
        }
    }

    pub fn with_policy(policy: SchedulingPolicy) -> Self {
        Self {
            policy,
            ..Self::new()
        }
    }

    pub fn metrics(&self) -> Arc<SchedulerMetrics> {
        self.metrics.clone()
    }

    pub fn schedule(&self, task: ScheduledTask) -> Result<TaskId> {
        let id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let mut task = task;
        task.set_id(TaskId::new(id));
        task.set_scheduled_at(self.time.fetch_add(1, AtomicOrdering::SeqCst));
        self.task_queue.lock().unwrap().push(task.clone());
        self.metrics.tasks_scheduled.fetch_add(1, AtomicOrdering::Release);
        Ok(task.id())
    }

    pub fn schedule_event_driven(
        &self,
        source: CellId,
        event_payload: EventPayload,
        priority: TaskPriority,
    ) -> Result<TaskId> {
        let task = ScheduledTask::event_driven(source, event_payload, priority);
        self.schedule(task)
    }

    pub fn schedule_recurring(
        &self,
        source: CellId,
        event_payload: EventPayload,
        priority: TaskPriority,
        interval_ms: u64,
    ) -> Result<TaskId> {
        let task = ScheduledTask::recurring(source, event_payload, priority, interval_ms);
        self.schedule(task)
    }

    pub fn next_task(&self) -> Option<ScheduledTask> {
        let mut queue = self.task_queue.lock().unwrap();
        let task = queue.pop()?;
        if task.status() == TaskStatus::Cancelled {
            self.metrics.tasks_cancelled.fetch_add(1, AtomicOrdering::Release);
            return self.next_task();
        }
        self.active_tasks.lock().unwrap().insert(task.id(), task.clone());
        Some(task)
    }

    pub fn complete_task(&self, task_id: &TaskId) -> Result<()> {
        let task = self.active_tasks.lock().unwrap().remove(task_id)
            .ok_or_else(|| Error::Custom(format!("Task not active: {}", task_id)))?;
        self.completed_tasks.lock().unwrap().insert(task_id.clone(), task);
        self.metrics.tasks_completed.fetch_add(1, AtomicOrdering::Release);
        Ok(())
    }

    pub fn fail_task(&self, task_id: &TaskId, reason: impl Into<String>) -> Result<()> {
        let _task = self.active_tasks.lock().unwrap().remove(task_id)
            .ok_or_else(|| Error::Custom(format!("Task not active: {}", task_id)))?;
        self.failed_tasks.lock().unwrap().insert(task_id.clone(), reason.into());
        self.metrics.tasks_failed.fetch_add(1, AtomicOrdering::Release);
        Ok(())
    }

    pub fn cancel_task(&self, task_id: &TaskId) -> Result<()> {
        let mut queue = self.task_queue.lock().unwrap();
        let mut found = false;
        let mut temp = Vec::new();
        while let Some(task) = queue.pop() {
            if task.id() == *task_id {
                found = true;
            } else {
                temp.push(task);
            }
        }
        for task in temp {
            queue.push(task);
        }
        if !found {
            return Err(Error::Custom(format!("Task not found in queue: {}", task_id)));
        }
        self.metrics.tasks_cancelled.fetch_add(1, AtomicOrdering::Release);
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }

    pub fn active_count(&self) -> usize {
        self.active_tasks.lock().unwrap().len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed_tasks.lock().unwrap().len()
    }

    pub fn failed_count(&self) -> usize {
        self.failed_tasks.lock().unwrap().len()
    }

    pub fn clear(&self) {
        self.task_queue.lock().unwrap().clear();
        self.active_tasks.lock().unwrap().clear();
        self.completed_tasks.lock().unwrap().clear();
        self.failed_tasks.lock().unwrap().clear();
    }

    pub fn policy(&self) -> SchedulingPolicy {
        self.policy
    }

    pub fn set_policy(&mut self, policy: SchedulingPolicy) {
        self.policy = policy;
    }

    pub fn dump_state(&self) -> SchedulerState {
        SchedulerState {
            pending: self.pending_count(),
            active: self.active_count(),
            completed: self.completed_count(),
            failed: self.failed_count(),
            policy: self.policy,
            metrics: self.metrics.snapshot(),
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerState {
    pub pending: usize,
    pub active: usize,
    pub completed: usize,
    pub failed: usize,
    pub policy: SchedulingPolicy,
    pub metrics: MetricsSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskPriority;

    fn make_task(priority: TaskPriority) -> ScheduledTask {
        ScheduledTask::new(
            CellId::random(),
            EventPayload::new(vec![], "text/plain"),
            priority,
        )
    }

    #[test]
    fn test_schedule_and_next() {
        let s = Scheduler::new();
        let task = make_task(TaskPriority::Normal);
        let id = s.schedule(task).unwrap();
        assert_eq!(s.pending_count(), 1);
        let next = s.next_task().unwrap();
        assert_eq!(next.id(), id);
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let s = Scheduler::new();
        let low = s.schedule(make_task(TaskPriority::Low)).unwrap();
        let high = s.schedule(make_task(TaskPriority::High)).unwrap();
        let crit = s.schedule(make_task(TaskPriority::Critical)).unwrap();

        assert_eq!(s.next_task().unwrap().id(), crit);
        assert_eq!(s.next_task().unwrap().id(), high);
        assert_eq!(s.next_task().unwrap().id(), low);
    }

    #[test]
    fn test_complete_task() {
        let s = Scheduler::new();
        let id = s.schedule(make_task(TaskPriority::Normal)).unwrap();
        let task = s.next_task().unwrap();
        assert_eq!(task.id(), id);
        assert_eq!(s.active_count(), 1);
        s.complete_task(&id).unwrap();
        assert_eq!(s.active_count(), 0);
        assert_eq!(s.completed_count(), 1);
    }

    #[test]
    fn test_fail_task() {
        let s = Scheduler::new();
        let id = s.schedule(make_task(TaskPriority::Normal)).unwrap();
        s.next_task().unwrap();
        s.fail_task(&id, "test failure").unwrap();
        assert_eq!(s.failed_count(), 1);
    }

    #[test]
    fn test_cancel_pending_task() {
        let s = Scheduler::new();
        let id = s.schedule(make_task(TaskPriority::Normal)).unwrap();
        assert_eq!(s.pending_count(), 1);
        s.cancel_task(&id).unwrap();
        assert_eq!(s.pending_count(), 0);
        assert!(s.next_task().is_none());
    }

    #[test]
    fn test_cancel_nonexistent() {
        let s = Scheduler::new();
        let result = s.cancel_task(&TaskId::new(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_all() {
        let s = Scheduler::new();
        s.schedule(make_task(TaskPriority::Normal)).unwrap();
        s.schedule(make_task(TaskPriority::High)).unwrap();
        s.schedule(make_task(TaskPriority::Low)).unwrap();
        assert_eq!(s.pending_count(), 3);
        s.clear();
        assert_eq!(s.pending_count(), 0);
    }

    #[test]
    fn test_event_driven_task() {
        let s = Scheduler::new();
        let source = CellId::random();
        let payload = EventPayload::new(vec![1, 2, 3], "application/octet-stream");
        let id = s.schedule_event_driven(source, payload, TaskPriority::Normal).unwrap();
        assert_eq!(s.pending_count(), 1);
        let task = s.next_task().unwrap();
        assert_eq!(task.id(), id);
    }

    #[test]
    fn test_recurring_task() {
        let s = Scheduler::new();
        let source = CellId::random();
        let payload = EventPayload::new(vec![], "text/plain");
        s.schedule_recurring(source, payload, TaskPriority::Low, 1000).unwrap();
        let task = s.next_task().unwrap();
        assert!(task.is_recurring());
        assert_eq!(task.interval_ms(), Some(1000));
    }

    #[test]
    fn test_dump_state() {
        let s = Scheduler::new();
        s.schedule(make_task(TaskPriority::Normal)).unwrap();
        let state = s.dump_state();
        assert_eq!(state.pending, 1);
        assert_eq!(state.active, 0);
        assert_eq!(state.policy, SchedulingPolicy::Priority);
    }

    #[test]
    fn test_empty_queue_returns_none() {
        let s = Scheduler::new();
        assert!(s.next_task().is_none());
    }

    #[test]
    fn test_multiple_tasks_fifo_within_same_priority() {
        let s = Scheduler::new();
        let ids: Vec<TaskId> = (0..5).map(|_| {
            s.schedule(make_task(TaskPriority::Normal)).unwrap()
        }).collect();
        for expected_id in &ids {
            let next = s.next_task().unwrap();
            assert_eq!(&next.id(), expected_id);
        }
    }
}
