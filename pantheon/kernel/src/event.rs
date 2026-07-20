use crate::capability::CapabilityId;
use crate::cell_id::CellId;
use crate::event_id::EventId;
use crate::protocol::ProtocolRef;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Command,
    Event,
    Notification,
    Query,
    Response,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::Command => "command",
            EventKind::Event => "event",
            EventKind::Notification => "notification",
            EventKind::Query => "query",
            EventKind::Response => "response",
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub data: Vec<u8>,
    pub content_type: String,
}

impl EventPayload {
    pub fn new(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            data,
            content_type: content_type.into(),
        }
    }

    pub fn from_json<T: Serialize>(value: &T) -> Result<Self, crate::error::Error> {
        let data = serde_json::to_vec(value)
            .map_err(|e| crate::error::Error::SerializationError(e.to_string()))?;
        Ok(Self {
            data,
            content_type: "application/json".to_string(),
        })
    }

    pub fn to_json<'a, T: Deserialize<'a>>(&'a self) -> Result<T, crate::error::Error> {
        serde_json::from_slice(&self.data)
            .map_err(|e| crate::error::Error::DeserializationError(e.to_string()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub kind: EventKind,
    pub protocol: ProtocolRef,
    pub source: CellId,
    pub target: CapabilityId,
    pub payload: EventPayload,
    pub parent_event: Option<EventId>,
    pub is_non_deterministic: bool,
    pub ttl_seconds: u64,
}

impl Event {
    pub fn new(
        id: EventId,
        kind: EventKind,
        protocol: ProtocolRef,
        source: CellId,
        target: impl Into<CapabilityId>,
        payload: EventPayload,
    ) -> Self {
        Self {
            id,
            kind,
            protocol,
            source,
            target: target.into(),
            payload,
            parent_event: None,
            is_non_deterministic: false,
            ttl_seconds: 3600,
        }
    }

    pub fn with_parent(mut self, parent: EventId) -> Self {
        self.parent_event = Some(parent);
        self
    }

    pub fn with_non_deterministic(mut self, value: bool) -> Self {
        self.is_non_deterministic = value;
        self
    }

    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = seconds;
        self
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} from {} to {} via {}",
            self.id, self.kind, self.source, self.target, self.protocol
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_id::CellId;
    use crate::event_id::EventId;
    use crate::protocol::ProtocolRef;
    use crate::types::{LamportTimestamp, SequenceNumber};

    #[test]
    fn test_event_creation() {
        let cell = CellId::random();
        let eid = EventId::new(cell, LamportTimestamp::new(1), SequenceNumber::new(0));
        let event = Event::new(
            eid,
            EventKind::Command,
            ProtocolRef::new("pantheon.test.v1"),
            cell,
            "test.capability",
            EventPayload::new(vec![1, 2, 3], "application/octet-stream"),
        );

        assert_eq!(event.id, eid);
        assert_eq!(event.kind, EventKind::Command);
        assert_eq!(event.source, cell);
    }

    #[test]
    fn test_event_payload_json_roundtrip() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestMsg {
            value: u64,
            name: String,
        }

        let msg = TestMsg {
            value: 42,
            name: "test".to_string(),
        };

        let payload = EventPayload::from_json(&msg).unwrap();
        let decoded: TestMsg = payload.to_json().unwrap();

        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_event_display() {
        let cell = CellId::random();
        let eid = EventId::new(cell, LamportTimestamp::new(1), SequenceNumber::new(0));
        let event = Event::new(
            eid,
            EventKind::Notification,
            ProtocolRef::new("pantheon.test.v1"),
            cell,
            "test.target",
            EventPayload::new(vec![], "text/plain"),
        );

        let s = event.to_string();
        assert!(s.contains("notification"));
        assert!(s.contains("pantheon.test.v1"));
    }

    #[test]
    fn test_event_parent_chain() {
        let cell = CellId::random();
        let parent_eid = EventId::new(cell, LamportTimestamp::new(1), SequenceNumber::new(0));
        let child_eid = EventId::new(cell, LamportTimestamp::new(2), SequenceNumber::new(1));

        let child = Event::new(
            child_eid,
            EventKind::Event,
            ProtocolRef::new("pantheon.test.v1"),
            cell,
            "test.target",
            EventPayload::new(vec![], "text/plain"),
        )
        .with_parent(parent_eid);

        assert_eq!(child.parent_event, Some(parent_eid));
    }

    #[test]
    fn test_event_non_deterministic_flag() {
        let cell = CellId::random();
        let eid = EventId::new(cell, LamportTimestamp::new(1), SequenceNumber::new(0));
        let event = Event::new(
            eid,
            EventKind::Event,
            ProtocolRef::new("pantheon.test.v1"),
            cell,
            "test.target",
            EventPayload::new(vec![], "text/plain"),
        )
        .with_non_deterministic(true);

        assert!(event.is_non_deterministic);
    }
}
