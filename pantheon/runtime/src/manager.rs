use std::collections::HashMap;
use std::time::Instant;
use pantheon_kernel::cell::{Cell, CellSpec, CellState, CellType};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::cell::CellStatus;
use pantheon_kernel::error::{Error, Result};
use pantheon_kernel::event::EventPayload;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub cell_id: CellId,
    pub command: String,
    pub output: String,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub total_cells_created: u64,
    pub total_cells_destroyed: u64,
    pub cells_alive: usize,
    pub tasks_executed: u64,
    pub tasks_failed: u64,
    pub total_runtime_ms: u64,
    pub total_memory_bytes: u64,
    pub total_cpu_millicores: u64,
}

impl RuntimeStats {
    pub fn new() -> Self {
        Self {
            total_cells_created: 0,
            total_cells_destroyed: 0,
            cells_alive: 0,
            tasks_executed: 0,
            tasks_failed: 0,
            total_runtime_ms: 0,
            total_memory_bytes: 0,
            total_cpu_millicores: 0,
        }
    }
}

pub struct CellManager {
    cells: HashMap<CellId, Cell>,
    #[allow(dead_code)]
    start_time: Instant,
    stats: RuntimeStats,
}

impl CellManager {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            start_time: Instant::now(),
            stats: RuntimeStats::new(),
        }
    }

    pub fn create_cell(&mut self, spec: CellSpec) -> Result<CellId> {
        let id = CellId::random();
        let cell = Cell::new(id, spec);
        self.cells.insert(id, cell);
        self.stats.total_cells_created += 1;
        Ok(id)
    }

    pub fn create_cell_with_id(&mut self, id: CellId, spec: CellSpec) -> Result<CellId> {
        if self.cells.contains_key(&id) {
            return Err(Error::Custom(format!("Cell already exists: {}", id)));
        }
        let cell = Cell::new(id, spec);
        self.cells.insert(id, cell);
        self.stats.total_cells_created += 1;
        Ok(id)
    }

    pub fn destroy_cell(&mut self, id: &CellId) -> Result<()> {
        let cell = self.cells.get_mut(id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", id))
        })?;
        // Walk through the transition chain to reach Destroyed
        match cell.state() {
            CellState::Created | CellState::Running | CellState::Suspended | CellState::Blocked => {
                cell.transition_to(CellState::Terminated)?;
                cell.transition_to(CellState::Retired)?;
                cell.transition_to(CellState::Destroyed)?;
            }
            CellState::Terminated => {
                cell.transition_to(CellState::Retired)?;
                cell.transition_to(CellState::Destroyed)?;
            }
            CellState::Retired => {
                cell.transition_to(CellState::Destroyed)?;
            }
            CellState::Destroyed => { /* already destroyed */ }
            CellState::Null => {
                return Err(Error::Custom(format!("Cell {} is in Null state", id)));
            }
        }
        self.cells.remove(id);
        self.stats.total_cells_destroyed += 1;
        Ok(())
    }

    pub fn get_cell(&self, id: &CellId) -> Option<&Cell> {
        self.cells.get(id)
    }

    pub fn get_cell_mut(&mut self, id: &CellId) -> Option<&mut Cell> {
        self.cells.get_mut(id)
    }

    pub fn start_cell(&mut self, id: &CellId) -> Result<()> {
        let cell = self.cells.get_mut(id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", id))
        })?;
        cell.transition_to(CellState::Running)
    }

    pub fn stop_cell(&mut self, id: &CellId) -> Result<()> {
        let cell = self.cells.get_mut(id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", id))
        })?;
        cell.transition_to(CellState::Terminated)
    }

    pub fn suspend_cell(&mut self, id: &CellId) -> Result<()> {
        let cell = self.cells.get_mut(id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", id))
        })?;
        cell.transition_to(CellState::Suspended)
    }

    pub fn resume_cell(&mut self, id: &CellId) -> Result<()> {
        let cell = self.cells.get_mut(id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", id))
        })?;
        let current = cell.state();
        if current != CellState::Suspended {
            return Err(Error::Custom(format!(
                "Cannot resume cell {} in state {:?}",
                id, current
            )));
        }
        cell.transition_to(CellState::Running)
    }

    pub fn alive_cells(&self) -> Vec<CellStatus> {
        self.cells
            .values()
            .filter(|c| c.state().is_active())
            .map(|c| c.status())
            .collect()
    }

    pub fn cells_by_type(&self, cell_type: CellType) -> Vec<&Cell> {
        self.cells
            .values()
            .filter(|c| c.cell_type() == cell_type)
            .collect()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn cell_ids(&self) -> Vec<CellId> {
        self.cells.keys().copied().collect()
    }

    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut RuntimeStats {
        &mut self.stats
    }

    pub fn snapshot(&self) -> HashMap<CellId, CellState> {
        self.cells
            .iter()
            .map(|(id, cell)| (*id, cell.state()))
            .collect()
    }
}

pub struct TaskExecutor {
    cell_manager: CellManager,
}

impl TaskExecutor {
    pub fn new(cell_manager: CellManager) -> Self {
        Self { cell_manager }
    }

    pub fn into_manager(self) -> CellManager {
        self.cell_manager
    }

    pub fn execute(&mut self, cell_id: &CellId, command: &str, payload: &EventPayload) -> Result<TaskResult> {
        let cell = self.cell_manager.get_cell_mut(cell_id).ok_or_else(|| {
            Error::Custom(format!("Cell not found: {}", cell_id))
        })?;

        if !cell.state().is_active() {
            return Err(Error::Custom(format!(
                "Cell {} is not active (state: {:?})",
                cell_id,
                cell.state()
            )));
        }

        let start = Instant::now();
        cell.increment_event_count();

        let output = format!(
            "Cell {} executed '{}' ({} bytes)",
            cell_id,
            command,
            payload.as_bytes().len()
        );

        let duration_ms = start.elapsed().as_millis() as u64;
        cell.update_resource_usage(
            payload.as_bytes().len() as u64,
            10,
            duration_ms,
        );

        let result = TaskResult {
            cell_id: *cell_id,
            command: command.to_string(),
            output: output.clone(),
            duration_ms,
            success: true,
        };

        self.cell_manager.stats.tasks_executed += 1;
        self.cell_manager.stats.total_runtime_ms += duration_ms;
        self.cell_manager.stats.total_memory_bytes += payload.as_bytes().len() as u64;

        Ok(result)
    }

    pub fn execute_simulated(&mut self, cell_id: &CellId, command: &str) -> Result<TaskResult> {
        let payload = EventPayload::new(
            format!(r#"{{"command":"{}"}}"#, command).into_bytes(),
            "application/json",
        );
        self.execute(cell_id, command, &payload)
    }

    pub fn stats(&self) -> &RuntimeStats {
        self.cell_manager.stats()
    }

    pub fn cell_manager(&self) -> &CellManager {
        &self.cell_manager
    }

    pub fn cell_manager_mut(&mut self) -> &mut CellManager {
        &mut self.cell_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worker_spec() -> CellSpec {
        CellSpec::new(CellType::Worker)
    }

    #[test]
    fn test_create_and_start_cell() {
        let mut mgr = CellManager::new();
        let id = mgr.create_cell(worker_spec()).unwrap();
        assert_eq!(mgr.cell_count(), 1);

        mgr.start_cell(&id).unwrap();
        let cell = mgr.get_cell(&id).unwrap();
        assert_eq!(cell.state(), CellState::Running);
    }

    #[test]
    fn test_cell_lifecycle() {
        let mut mgr = CellManager::new();
        let id = mgr.create_cell(worker_spec()).unwrap();

        mgr.start_cell(&id).unwrap();
        mgr.suspend_cell(&id).unwrap();
        mgr.resume_cell(&id).unwrap();
        mgr.stop_cell(&id).unwrap();

        let cell = mgr.get_cell(&id).unwrap();
        assert_eq!(cell.state(), CellState::Terminated);
    }

    #[test]
    fn test_destroy_cell() {
        let mut mgr = CellManager::new();
        let id = mgr.create_cell(worker_spec()).unwrap();
        mgr.start_cell(&id).unwrap();
        mgr.stop_cell(&id).unwrap();
        mgr.destroy_cell(&id).unwrap();
        assert_eq!(mgr.cell_count(), 0);
    }

    #[test]
    fn test_create_cell_with_id_duplicate() {
        let mut mgr = CellManager::new();
        let id = CellId::random();
        mgr.create_cell_with_id(id, worker_spec()).unwrap();
        let result = mgr.create_cell_with_id(id, worker_spec());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition() {
        let mut mgr = CellManager::new();
        let id = mgr.create_cell(worker_spec()).unwrap();
        // Can't go from Created to Suspended
        let result = mgr.suspend_cell(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_alive_cells() {
        let mut mgr = CellManager::new();
        let id1 = mgr.create_cell(worker_spec()).unwrap();
        let _id2 = mgr.create_cell(worker_spec()).unwrap();

        mgr.start_cell(&id1).unwrap();

        let alive = mgr.alive_cells();
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[0].id, id1);
    }

    #[test]
    fn test_execute_task() {
        let mgr = CellManager::new();
        let mut exec = TaskExecutor::new(mgr);
        let id = exec.cell_manager_mut().create_cell(worker_spec()).unwrap();
        exec.cell_manager_mut().start_cell(&id).unwrap();

        let result = exec.execute_simulated(&id, "test-command").unwrap();
        assert!(result.success);
        assert_eq!(result.command, "test-command");
    }

    #[test]
    fn test_execute_on_inactive_cell_fails() {
        let mgr = CellManager::new();
        let mut exec = TaskExecutor::new(mgr);
        let id = exec.cell_manager_mut().create_cell(worker_spec()).unwrap();
        // Cell is Created, not Running — should fail
        let result = exec.execute_simulated(&id, "fail");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_stats() {
        let mgr = CellManager::new();
        let mut exec = TaskExecutor::new(mgr);
        let id = exec.cell_manager_mut().create_cell(worker_spec()).unwrap();
        exec.cell_manager_mut().start_cell(&id).unwrap();

        exec.execute_simulated(&id, "cmd1").unwrap();
        exec.execute_simulated(&id, "cmd2").unwrap();

        let stats = exec.stats();
        assert_eq!(stats.tasks_executed, 2);
    }

    #[test]
    fn test_cells_by_type() {
        let mut mgr = CellManager::new();
        let _id1 = mgr.create_cell(CellSpec::new(CellType::Coordinator)).unwrap();
        let _id2 = mgr.create_cell(worker_spec()).unwrap();

        let workers = mgr.cells_by_type(CellType::Worker);
        assert_eq!(workers.len(), 1);

        let coords = mgr.cells_by_type(CellType::Coordinator);
        assert_eq!(coords.len(), 1);
    }

    #[test]
    fn test_snapshot() {
        let mut mgr = CellManager::new();
        let id = mgr.create_cell(worker_spec()).unwrap();
        mgr.start_cell(&id).unwrap();

        let snap = mgr.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[&id], CellState::Running);
    }
}
