use pantheon_kernel::error::Result;
use pantheon_kernel::event::{Event, EventKind};
use pantheon_kernel::event_id::EventId;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event: Event,
    pub offset: u64,
    pub timestamp_ns: u128,
}

impl EventRecord {
    pub fn new(event: Event, offset: u64) -> Self {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self {
            event,
            offset,
            timestamp_ns,
        }
    }
}

#[derive(Debug)]
pub struct EventLog {
    partition_id: String,
    records: VecDeque<EventRecord>,
    next_offset: u64,
    max_size: usize,
    byte_size: usize,
    retained_for_compaction: Vec<u64>,
}

impl EventLog {
    pub fn new(partition_id: impl Into<String>) -> Self {
        Self {
            partition_id: partition_id.into(),
            records: VecDeque::new(),
            next_offset: 0,
            max_size: 100_000,
            byte_size: 0,
            retained_for_compaction: Vec::new(),
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn append(&mut self, event: Event) -> Result<EventId> {
        let offset = self.next_offset;

        if self.records.len() >= self.max_size {
            if let Some(evicted) = self.records.pop_front() {
                self.byte_size = self.byte_size.saturating_sub(evicted.event.payload.as_bytes().len() + 256);
            }
        }

        let record = EventRecord::new(event, offset);
        let event_id = record.event.id;
        self.byte_size += record.event.payload.as_bytes().len() + 256;
        self.records.push_back(record);
        self.next_offset += 1;

        Ok(event_id)
    }

    pub fn read_from(&self, offset: u64, limit: Option<usize>) -> Vec<&EventRecord> {
        let start = offset as usize;
        if start >= self.records.len() {
            return Vec::new();
        }
        let limit = limit.unwrap_or(self.records.len());
        self.records
            .iter()
            .skip(start)
            .take(limit)
            .collect()
    }

    pub fn read_range(&self, start_offset: u64, end_offset: u64) -> Vec<&EventRecord> {
        let start = start_offset as usize;
        let end = (end_offset as usize).min(self.records.len());
        if start >= end {
            return Vec::new();
        }
        self.records.iter().skip(start).take(end - start).collect()
    }

    pub fn len(&self) -> u64 {
        self.next_offset
    }

    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    pub fn last_offset(&self) -> Option<u64> {
        if self.next_offset == 0 {
            None
        } else {
            Some(self.next_offset - 1)
        }
    }

    pub fn compact(&mut self) -> u64 {
        if self.retained_for_compaction.is_empty() {
            return 0;
        }

        let compact_set: std::collections::HashSet<u64> =
            self.retained_for_compaction.iter().copied().collect();
        let before = self.records.len();

        self.records.retain(|r| compact_set.contains(&r.offset));
        self.retained_for_compaction.clear();

        (before - self.records.len()) as u64
    }

    pub fn mark_retained(&mut self, offset: u64) {
        self.retained_for_compaction.push(offset);
    }

    pub fn filter_by_kind(&self, kind: EventKind) -> Vec<&EventRecord> {
        self.records
            .iter()
            .filter(|r| r.event.kind == kind)
            .collect()
    }

    pub fn filter_by_source(&self, source: &pantheon_kernel::cell_id::CellId) -> Vec<&EventRecord> {
        self.records
            .iter()
            .filter(|r| r.event.source == *source)
            .collect()
    }

    pub fn filter_by_protocol(&self, protocol_name: &str) -> Vec<&EventRecord> {
        self.records
            .iter()
            .filter(|r| r.event.protocol.name == protocol_name)
            .collect()
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
    fn test_append_and_read() {
        let mut log = EventLog::new("test-partition");
        let cell = CellId::random();

        for i in 0..5 {
            log.append(make_event(cell, EventKind::Event, i)).unwrap();
        }

        assert_eq!(log.len(), 5);
        let records = log.read_from(0, None);
        assert_eq!(records.len(), 5);
    }

    #[test]
    fn test_read_range() {
        let mut log = EventLog::new("test");
        let cell = CellId::random();

        for i in 0..10 {
            log.append(make_event(cell, EventKind::Event, i)).unwrap();
        }

        let records = log.read_range(3, 7);
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].offset, 3);
        assert_eq!(records[3].offset, 6);
    }

    #[test]
    fn test_max_size_eviction() {
        let mut log = EventLog::new("test").with_max_size(3);
        let cell = CellId::random();

        for i in 0..5 {
            log.append(make_event(cell, EventKind::Event, i)).unwrap();
        }

        assert_eq!(log.len(), 5);
        let records = log.read_from(0, None);
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].offset, 2);
    }

    #[test]
    fn test_empty_log() {
        let log = EventLog::new("empty");
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(log.last_offset().is_none());
    }

    #[test]
    fn test_filter_by_kind() {
        let mut log = EventLog::new("test");
        let cell = CellId::random();

        log.append(make_event(cell, EventKind::Command, 0)).unwrap();
        log.append(make_event(cell, EventKind::Event, 1)).unwrap();
        log.append(make_event(cell, EventKind::Notification, 2)).unwrap();
        log.append(make_event(cell, EventKind::Command, 3)).unwrap();

        assert_eq!(log.filter_by_kind(EventKind::Command).len(), 2);
        assert_eq!(log.filter_by_kind(EventKind::Notification).len(), 1);
    }

    #[test]
    fn test_byte_size_tracking() {
        let mut log = EventLog::new("test");
        let cell = CellId::random();

        for i in 0..10 {
            log.append(make_event(cell, EventKind::Event, i)).unwrap();
        }

        assert!(log.byte_size() > 0);
    }
}
