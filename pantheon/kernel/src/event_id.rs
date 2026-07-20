use crate::cell_id::CellId;
use crate::types::{LamportTimestamp, SequenceNumber};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId {
    pub origin: CellId,
    pub timestamp: LamportTimestamp,
    pub sequence: SequenceNumber,
}

impl EventId {
    pub const fn new(origin: CellId, timestamp: LamportTimestamp, sequence: SequenceNumber) -> Self {
        Self {
            origin,
            timestamp,
            sequence,
        }
    }

    pub fn origin(&self) -> &CellId {
        &self.origin
    }

    pub fn timestamp(&self) -> LamportTimestamp {
        self.timestamp
    }

    pub fn sequence(&self) -> SequenceNumber {
        self.sequence
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "evt://{}/{}:{}", self.origin, self.timestamp, self.sequence)
    }
}

impl fmt::Debug for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventId({})", self)
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self {
            origin: CellId::zero(),
            timestamp: LamportTimestamp::new(0),
            sequence: SequenceNumber::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_creation() {
        let cell = CellId::random();
        let ts = LamportTimestamp::new(42);
        let seq = SequenceNumber::new(7);
        let eid = EventId::new(cell, ts, seq);

        assert_eq!(eid.origin(), &cell);
        assert_eq!(eid.timestamp(), ts);
        assert_eq!(eid.sequence(), seq);
    }

    #[test]
    fn test_event_id_display() {
        let cell = CellId::zero();
        let ts = LamportTimestamp::new(1);
        let seq = SequenceNumber::new(2);
        let eid = EventId::new(cell, ts, seq);
        let s = eid.to_string();
        assert!(s.contains("evt://"));
        assert!(s.contains("t1"));
        assert!(s.contains("seq2"));
    }

    #[test]
    fn test_event_id_ordering() {
        let cell = CellId::zero();
        let eid1 = EventId::new(cell, LamportTimestamp::new(1), SequenceNumber::new(0));
        let eid2 = EventId::new(cell, LamportTimestamp::new(2), SequenceNumber::new(0));
        assert!(eid1 < eid2);
    }
}
