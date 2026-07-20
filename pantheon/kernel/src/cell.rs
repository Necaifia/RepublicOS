use crate::cell_id::CellId;
use crate::capability::CapabilityId;
use crate::error::{Error, Result};
use crate::types::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CellType {
    Worker,
    Coordinator,
    Service,
    Agent,
    Gateway,
    Mirror,
}

impl CellType {
    pub fn can_have_children(&self) -> bool {
        matches!(self, CellType::Coordinator | CellType::Service | CellType::Agent)
    }

    pub fn is_persistent(&self) -> bool {
        !matches!(self, CellType::Worker | CellType::Mirror)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CellType::Worker => "worker",
            CellType::Coordinator => "coordinator",
            CellType::Service => "service",
            CellType::Agent => "agent",
            CellType::Gateway => "gateway",
            CellType::Mirror => "mirror",
        }
    }
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for CellType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "worker" => CellType::Worker,
            "coordinator" => CellType::Coordinator,
            "service" => CellType::Service,
            "agent" => CellType::Agent,
            "gateway" => CellType::Gateway,
            "mirror" => CellType::Mirror,
            _ => CellType::Worker,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CellState {
    Null,
    Created,
    Running,
    Suspended,
    Blocked,
    Terminated,
    Retired,
    Destroyed,
}

impl CellState {
    pub fn can_transition_to(&self, target: &CellState) -> bool {
        matches!(
            (self, target),
            (CellState::Null, CellState::Created)
                | (CellState::Created, CellState::Running)
                | (CellState::Created, CellState::Terminated)
                | (CellState::Running, CellState::Suspended)
                | (CellState::Running, CellState::Blocked)
                | (CellState::Running, CellState::Terminated)
                | (CellState::Suspended, CellState::Running)
                | (CellState::Suspended, CellState::Terminated)
                | (CellState::Blocked, CellState::Running)
                | (CellState::Blocked, CellState::Terminated)
                | (CellState::Terminated, CellState::Retired)
                | (CellState::Retired, CellState::Destroyed)
                | (CellState::Destroyed, CellState::Created)
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, CellState::Running | CellState::Blocked)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, CellState::Terminated | CellState::Retired | CellState::Destroyed)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CellState::Null => "null",
            CellState::Created => "created",
            CellState::Running => "running",
            CellState::Suspended => "suspended",
            CellState::Blocked => "blocked",
            CellState::Terminated => "terminated",
            CellState::Retired => "retired",
            CellState::Destroyed => "destroyed",
        }
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSpec {
    pub cell_type: CellType,
    pub parent: Option<CellId>,
    pub capabilities: HashSet<CapabilityId>,
    pub code_hash: ContentHash,
    pub max_children: u32,
    pub max_concurrent_tasks: u32,
    pub memory_limit_bytes: u64,
    pub cpu_millicores: u64,
    pub max_lifetime_seconds: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl CellSpec {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            parent: None,
            capabilities: HashSet::new(),
            code_hash: ContentHash::default(),
            max_children: 10,
            max_concurrent_tasks: 5,
            memory_limit_bytes: 64 * 1024 * 1024,
            cpu_millicores: 100,
            max_lifetime_seconds: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent: CellId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_capability(mut self, cap: CapabilityId) -> Self {
        self.capabilities.insert(cap);
        self
    }

    pub fn with_memory_limit(mut self, bytes: u64) -> Self {
        self.memory_limit_bytes = bytes;
        self
    }

    pub fn with_cpu(mut self, millicores: u64) -> Self {
        self.cpu_millicores = millicores;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CellStatus {
    pub id: CellId,
    pub cell_type: CellType,
    pub state: CellState,
    pub parent: Option<CellId>,
    pub children: Vec<CellId>,
    pub capabilities: HashSet<CapabilityId>,
    pub created_at_ts: u64,
    pub running_time_ms: u64,
    pub memory_used_bytes: u64,
    pub cpu_used_millicores: u64,
    pub event_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub struct Cell {
    id: CellId,
    cell_type: CellType,
    state: CellState,
    parent: Option<CellId>,
    children: Vec<CellId>,
    spec: CellSpec,
    capabilities: HashSet<CapabilityId>,
    created_at_ts: u64,
    modified_at_ts: u64,
    event_count: u64,
    error_count: u64,
    running_time_ms: u64,
    memory_used_bytes: u64,
    cpu_used_millicores: u64,
}

impl Cell {
    pub fn new(id: CellId, spec: CellSpec) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            cell_type: spec.cell_type,
            state: CellState::Created,
            parent: spec.parent,
            children: Vec::new(),
            capabilities: spec.capabilities.clone(),
            spec,
            created_at_ts: now,
            modified_at_ts: now,
            event_count: 0,
            error_count: 0,
            running_time_ms: 0,
            memory_used_bytes: 0,
            cpu_used_millicores: 0,
        }
    }

    pub fn id(&self) -> &CellId {
        &self.id
    }

    pub fn cell_type(&self) -> CellType {
        self.cell_type
    }

    pub fn state(&self) -> CellState {
        self.state
    }

    pub fn parent(&self) -> Option<&CellId> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> &[CellId] {
        &self.children
    }

    pub fn spec(&self) -> &CellSpec {
        &self.spec
    }

    pub fn capabilities(&self) -> &HashSet<CapabilityId> {
        &self.capabilities
    }

    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    pub fn error_count(&self) -> u64 {
        self.error_count
    }

    pub fn add_child(&mut self, child_id: CellId) -> Result<()> {
        if self.children.len() as u32 >= self.spec.max_children {
            return Err(Error::ResourceExhausted(format!(
                "Cell {} has reached max children ({})",
                self.id, self.spec.max_children
            )));
        }
        self.children.push(child_id);
        self.modified_at_ts = self.current_time();
        Ok(())
    }

    pub fn remove_child(&mut self, child_id: &CellId) {
        self.children.retain(|c| c != child_id);
        self.modified_at_ts = self.current_time();
    }

    pub fn transition_to(&mut self, target: CellState) -> Result<()> {
        if !self.state.can_transition_to(&target) {
            return Err(Error::InvalidTransition {
                from: self.state.as_str().to_string(),
                to: target.as_str().to_string(),
            });
        }
        self.state = target;
        self.modified_at_ts = self.current_time();
        Ok(())
    }

    pub fn increment_event_count(&mut self) {
        self.event_count += 1;
    }

    pub fn increment_error_count(&mut self) {
        self.error_count += 1;
    }

    pub fn update_resource_usage(&mut self, memory_bytes: u64, cpu_millicores: u64, time_ms: u64) {
        self.memory_used_bytes = memory_bytes;
        self.cpu_used_millicores = cpu_millicores;
        self.running_time_ms += time_ms;
    }

    pub fn status(&self) -> CellStatus {
        CellStatus {
            id: self.id,
            cell_type: self.cell_type,
            state: self.state,
            parent: self.parent,
            children: self.children.clone(),
            capabilities: self.capabilities.clone(),
            created_at_ts: self.created_at_ts,
            running_time_ms: self.running_time_ms,
            memory_used_bytes: self.memory_used_bytes,
            cpu_used_millicores: self.cpu_used_millicores,
            event_count: self.event_count,
            error_count: self.error_count,
        }
    }

    fn current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spec() -> CellSpec {
        CellSpec::new(CellType::Worker)
    }

    #[test]
    fn test_cell_creation() {
        let id = CellId::random();
        let cell = Cell::new(id, test_spec());
        assert_eq!(cell.state(), CellState::Created);
        assert_eq!(cell.cell_type(), CellType::Worker);
        assert_eq!(cell.id(), &id);
    }

    #[test]
    fn test_cell_transition_valid() {
        let id = CellId::random();
        let mut cell = Cell::new(id, test_spec());

        assert!(cell.transition_to(CellState::Running).is_ok());
        assert_eq!(cell.state(), CellState::Running);

        assert!(cell.transition_to(CellState::Suspended).is_ok());
        assert_eq!(cell.state(), CellState::Suspended);

        assert!(cell.transition_to(CellState::Running).is_ok());
        assert_eq!(cell.state(), CellState::Running);

        assert!(cell.transition_to(CellState::Terminated).is_ok());
        assert_eq!(cell.state(), CellState::Terminated);
    }

    #[test]
    fn test_cell_transition_invalid() {
        let id = CellId::random();
        let mut cell = Cell::new(id, test_spec());

        // Can't go from Created to Suspended
        assert!(cell.transition_to(CellState::Suspended).is_err());

        // Can't go from Terminated to Running
        cell.transition_to(CellState::Running).unwrap();
        cell.transition_to(CellState::Terminated).unwrap();
        assert!(cell.transition_to(CellState::Running).is_err());
    }

    #[test]
    fn test_add_child() {
        let parent_id = CellId::random();
        let child_id = CellId::random();
        let mut spec = CellSpec::new(CellType::Coordinator);
        spec.max_children = 5;
        let mut cell = Cell::new(parent_id, spec);

        assert!(cell.add_child(child_id).is_ok());
        assert_eq!(cell.children().len(), 1);
        assert_eq!(cell.children()[0], child_id);
    }

    #[test]
    fn test_add_child_exceeds_limit() {
        let parent_id = CellId::random();
        let mut spec = CellSpec::new(CellType::Coordinator);
        spec.max_children = 2;
        let mut cell = Cell::new(parent_id, spec);

        assert!(cell.add_child(CellId::random()).is_ok());
        assert!(cell.add_child(CellId::random()).is_ok());
        assert!(cell.add_child(CellId::random()).is_err());
    }

    #[test]
    fn test_remove_child() {
        let parent_id = CellId::random();
        let child_id = CellId::random();
        let mut cell = Cell::new(parent_id, CellSpec::new(CellType::Coordinator));

        cell.add_child(child_id).unwrap();
        assert_eq!(cell.children().len(), 1);

        cell.remove_child(&child_id);
        assert!(cell.children().is_empty());
    }

    #[test]
    fn test_cell_type_properties() {
        assert!(CellType::Coordinator.can_have_children());
        assert!(!CellType::Worker.can_have_children());
        assert!(CellType::Service.is_persistent());
        assert!(!CellType::Worker.is_persistent());
    }

    #[test]
    fn test_cell_state_transition_table() {
        let _all_states = [
            CellState::Null,
            CellState::Created,
            CellState::Running,
            CellState::Suspended,
            CellState::Blocked,
            CellState::Terminated,
            CellState::Retired,
            CellState::Destroyed,
        ];

        assert!(CellState::Null.can_transition_to(&CellState::Created));
        assert!(CellState::Created.can_transition_to(&CellState::Running));
        assert!(CellState::Running.can_transition_to(&CellState::Suspended));
        assert!(CellState::Suspended.can_transition_to(&CellState::Running));
        assert!(CellState::Terminated.can_transition_to(&CellState::Retired));
        assert!(CellState::Retired.can_transition_to(&CellState::Destroyed));
        assert!(CellState::Destroyed.can_transition_to(&CellState::Created));

        // Invalid transitions
        assert!(!CellState::Null.can_transition_to(&CellState::Running));
        assert!(!CellState::Created.can_transition_to(&CellState::Null));
        assert!(!CellState::Running.can_transition_to(&CellState::Created));
        assert!(!CellState::Terminated.can_transition_to(&CellState::Running));
    }

    #[test]
    fn test_cell_status() {
        let id = CellId::random();
        let cell = Cell::new(id, test_spec());
        let status = cell.status();
        assert_eq!(status.id, id);
        assert_eq!(status.state, CellState::Created);
        assert_eq!(status.event_count, 0);
    }

    #[test]
    fn test_increment_counters() {
        let id = CellId::random();
        let mut cell = Cell::new(id, test_spec());
        cell.increment_event_count();
        cell.increment_event_count();
        cell.increment_error_count();
        assert_eq!(cell.event_count(), 2);
        assert_eq!(cell.error_count(), 1);
    }

    #[test]
    fn test_cell_spec_builder() {
        let spec = CellSpec::new(CellType::Agent)
            .with_memory_limit(256 * 1024 * 1024)
            .with_cpu(500)
            .with_capability(CapabilityId::new("memory.read"));

        assert_eq!(spec.memory_limit_bytes, 256 * 1024 * 1024);
        assert_eq!(spec.cpu_millicores, 500);
        assert!(spec.capabilities.contains(&CapabilityId::new("memory.read")));
    }
}
