use std::collections::HashMap;
use std::sync::Mutex;
use pantheon_kernel::cell::CellState;
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::Result;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CellInstance {
    pub id: CellId,
    pub name: String,
    pub state: CellState,
    pub created_at_ns: u128,
}

pub struct RuntimeManager {
    cells: Mutex<HashMap<CellId, CellInstance>>,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            cells: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_cell(&self, name: &str) -> CellId {
        let id = CellId::random();
        let cell = CellInstance {
            id,
            name: name.to_string(),
            state: CellState::Running,
            created_at_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        };
        self.cells.lock().unwrap().insert(id, cell);
        id
    }

    pub fn cell_count(&self) -> usize {
        self.cells.lock().unwrap().len()
    }

    pub fn execute_task(&self, _cell_id: &CellId, _task_name: &str) -> Result<String> {
        Ok(format!("Executed task '{}' on cell {}", _task_name, _cell_id))
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}
