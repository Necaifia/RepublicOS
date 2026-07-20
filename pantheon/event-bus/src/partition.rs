use crate::log::{EventLog, EventRecord};
use pantheon_kernel::error::{Error, Result};
use pantheon_kernel::event::Event;
use pantheon_kernel::event_id::EventId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionState {
    Unassigned,
    Follower,
    Candidate,
    Leader,
    Merging,
}

impl PartitionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PartitionState::Unassigned => "unassigned",
            PartitionState::Follower => "follower",
            PartitionState::Candidate => "candidate",
            PartitionState::Leader => "leader",
            PartitionState::Merging => "merging",
        }
    }
}

#[derive(Debug)]
pub struct Partition {
    id: String,
    log: EventLog,
    state: PartitionState,
    leader_id: Option<String>,
    consumer_groups: HashMap<String, Vec<String>>,
}

impl Partition {
    pub fn new(id: impl Into<String>) -> Self {
        let id_str: String = id.into();
        Self {
            id: id_str.clone(),
            log: EventLog::new(format!("{}-log", id_str)),
            state: PartitionState::Unassigned,
            leader_id: None,
            consumer_groups: HashMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn state(&self) -> PartitionState {
        self.state
    }

    pub fn set_state(&mut self, state: PartitionState) {
        self.state = state;
    }

    pub fn leader_id(&self) -> Option<&str> {
        self.leader_id.as_deref()
    }

    pub fn set_leader(&mut self, leader: Option<String>) {
        if leader.is_some() {
            self.state = PartitionState::Leader;
        } else {
            self.state = PartitionState::Follower;
        }
        self.leader_id = leader;
    }

    pub fn append(&mut self, event: Event) -> Result<EventId> {
        if self.state != PartitionState::Leader {
            return Err(Error::Custom(format!(
                "Partition {} is not leader (state: {})",
                self.id,
                self.state.as_str()
            )));
        }
        self.log.append(event)
    }

    pub fn read_from(&self, offset: u64, limit: Option<usize>) -> Vec<&EventRecord> {
        self.log.read_from(offset, limit)
    }

    pub fn read_range(&self, start: u64, end: u64) -> Vec<&EventRecord> {
        self.log.read_range(start, end)
    }

    pub fn log_len(&self) -> u64 {
        self.log.len()
    }

    pub fn last_offset(&self) -> Option<u64> {
        self.log.last_offset()
    }

    pub fn register_consumer(&mut self, group_id: String, consumer_id: String) {
        self.consumer_groups
            .entry(group_id)
            .or_default()
            .push(consumer_id);
    }

    pub fn unregister_consumer(&mut self, group_id: &str, consumer_id: &str) {
        if let Some(consumers) = self.consumer_groups.get_mut(group_id) {
            consumers.retain(|c| c != consumer_id);
        }
    }
}

#[derive(Debug)]
pub struct PartitionManager {
    partitions: HashMap<String, Partition>,
    next_partition_id: AtomicU64,
}

impl PartitionManager {
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            next_partition_id: AtomicU64::new(0),
        }
    }

    pub fn create_partition(&mut self) -> PartitionId {
        let id_num = self.next_partition_id.fetch_add(1, Ordering::SeqCst);
        let id = format!("partition-{}", id_num);
        let partition = Partition::new(&id);
        self.partitions.insert(id.clone(), partition);
        PartitionId(id_num)
    }

    pub fn get_partition(&self, id: &PartitionId) -> Result<&Partition> {
        let key = format!("partition-{}", id.0);
        self.partitions
            .get(&key)
            .ok_or_else(|| Error::PartitionNotFound(key))
    }

    pub fn get_partition_mut(&mut self, id: &PartitionId) -> Result<&mut Partition> {
        let key = format!("partition-{}", id.0);
        self.partitions
            .get_mut(&key)
            .ok_or_else(|| Error::PartitionNotFound(key))
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn all_partitions(&self) -> Vec<&Partition> {
        self.partitions.values().collect()
    }

    pub fn remove_partition(&mut self, id: &PartitionId) -> Result<Partition> {
        let key = format!("partition-{}", id.0);
        self.partitions
            .remove(&key)
            .ok_or_else(|| Error::PartitionNotFound(key))
    }

    pub fn partition_for_key(&self, key: &str) -> Option<PartitionId> {
        if self.partitions.is_empty() {
            return None;
        }
        let hash = calculate_hash(key);
        let idx = hash as usize % self.partitions.len();
        self.partitions.values().nth(idx).map(|p| {
            let num: u64 = p.id().strip_prefix("partition-").unwrap_or("0").parse().unwrap_or(0);
            PartitionId(num)
        })
    }
}

impl Default for PartitionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_hash(key: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartitionId(pub u64);

impl std::fmt::Display for PartitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "partition-{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pantheon_kernel::capability::CapabilityId;
    use pantheon_kernel::cell_id::CellId;
    use pantheon_kernel::event::{Event, EventKind, EventPayload};
    use pantheon_kernel::event_id::EventId;
    use pantheon_kernel::protocol::ProtocolRef;
    use pantheon_kernel::types::{LamportTimestamp, SequenceNumber};

    fn make_event(source: CellId, kind: EventKind, seq: u64) -> Event {
        let eid = EventId::new(source, LamportTimestamp::new(seq), SequenceNumber::new(seq));
        Event::new(
            eid,
            kind,
            ProtocolRef::new("pantheon.test.v1"),
            source,
            CapabilityId::new("test.cap"),
            EventPayload::new(vec![], "text/plain"),
        )
    }

    #[test]
    fn test_create_partition() {
        let mut pm = PartitionManager::new();
        let pid = pm.create_partition();
        let partition = pm.get_partition(&pid).unwrap();
        assert_eq!(partition.state(), PartitionState::Unassigned);
    }

    #[test]
    fn test_append_only_when_leader() {
        let mut pm = PartitionManager::new();
        let pid = pm.create_partition();
        let cell = CellId::random();

        let result = pm
            .get_partition_mut(&pid)
            .unwrap()
            .append(make_event(cell, EventKind::Event, 0));
        assert!(result.is_err());

        pm.get_partition_mut(&pid)
            .unwrap()
            .set_leader(Some("node-1".to_string()));

        let result = pm
            .get_partition_mut(&pid)
            .unwrap()
            .append(make_event(cell, EventKind::Event, 1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_partitions() {
        let mut pm = PartitionManager::new();
        let p1 = pm.create_partition();
        let p2 = pm.create_partition();
        pm.create_partition();

        assert_eq!(pm.partition_count(), 3);
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_remove_partition() {
        let mut pm = PartitionManager::new();
        let pid = pm.create_partition();
        pm.remove_partition(&pid).unwrap();
        assert_eq!(pm.partition_count(), 0);
    }

    #[test]
    fn test_consumer_registration() {
        let mut pm = PartitionManager::new();
        let pid = pm.create_partition();
        let partition = pm.get_partition_mut(&pid).unwrap();

        partition.register_consumer("group-1".into(), "consumer-a".into());
        partition.register_consumer("group-1".into(), "consumer-b".into());
        partition.register_consumer("group-2".into(), "consumer-c".into());
    }
}
