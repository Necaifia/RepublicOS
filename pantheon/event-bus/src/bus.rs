use crate::log::EventLog;
use crate::partition::{PartitionId, PartitionManager};
use pantheon_kernel::capability::CapabilityId;
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::{Error, Result};
use pantheon_kernel::event::{Event, EventKind, EventPayload};
use pantheon_kernel::event_id::EventId;
use pantheon_kernel::lamport::LamportClock;
use pantheon_kernel::protocol::ProtocolRef;
use pantheon_kernel::types::SequenceNumber;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub type SubscriberCallback = Box<dyn Fn(&Event) -> Result<()> + Send + Sync>;

pub struct Subscription {
    id: u64,
    subscriber_id: String,
    capability_pattern: String,
    event_types: Vec<EventKind>,
    callback: SubscriberCallback,
    active: AtomicBool,
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("id", &self.id)
            .field("subscriber_id", &self.subscriber_id)
            .field("capability_pattern", &self.capability_pattern)
            .field("event_types", &self.event_types)
            .field("active", &self.active.load(Ordering::Acquire))
            .finish()
    }
}

impl Subscription {
    pub fn new(
        id: u64,
        subscriber_id: impl Into<String>,
        capability_pattern: impl Into<String>,
        event_types: Vec<EventKind>,
        callback: SubscriberCallback,
    ) -> Self {
        Self {
            id,
            subscriber_id: subscriber_id.into(),
            capability_pattern: capability_pattern.into(),
            event_types,
            callback,
            active: AtomicBool::new(true),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }

    pub fn matches(&self, event: &Event) -> bool {
        if !self.active.load(Ordering::Acquire) {
            return false;
        }
        if !self.event_types.is_empty() && !self.event_types.contains(&event.kind) {
            return false;
        }
        if !self.capability_pattern.is_empty() {
            let target = event.target.as_str();
            let pattern = if self.capability_pattern.ends_with('*') {
                &self.capability_pattern[..self.capability_pattern.len() - 1]
            } else {
                &self.capability_pattern
            };
            if !target.starts_with(pattern) {
                return false;
            }
        }
        true
    }

    pub fn deliver(&self, event: &Event) -> Result<()> {
        (self.callback)(event)
    }

    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }
}

pub struct EventBus {
    clock: LamportClock,
    sequence: AtomicU64,
    partitions: RwLock<PartitionManager>,
    subscriptions: Arc<RwLock<Vec<Subscription>>>,
    next_sub_id: AtomicU64,
    local_events: Arc<Mutex<EventLog>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("clock", &self.clock.peek())
            .field("sequence", &self.sequence.load(Ordering::Acquire))
            .field("subscriptions", &self.subscriptions.read().unwrap().len())
            .finish()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let pm = PartitionManager::new();
        Self {
            clock: LamportClock::new(0),
            sequence: AtomicU64::new(0),
            partitions: RwLock::new(pm),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            next_sub_id: AtomicU64::new(1),
            local_events: Arc::new(Mutex::new(EventLog::new("bus-local"))),
        }
    }

    pub fn create_partition(&self) -> PartitionId {
        self.partitions.write().unwrap().create_partition()
    }

    pub fn partition_for_key(&self, key: &str) -> Option<PartitionId> {
        self.partitions.read().unwrap().partition_for_key(key)
    }

    fn next_event_id(&self, source: CellId) -> EventId {
        let ts = self.clock.tick();
        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        EventId::new(source, ts, SequenceNumber::new(seq))
    }

    fn store_event(&self, event: &Event) -> Result<()> {
        let payload = EventPayload::from_json(event).map_err(|e| {
            Error::SerializationError(format!("Failed to serialize event: {}", e))
        })?;
        let stored = Event::new(
            event.id,
            event.kind,
            event.protocol.clone(),
            event.source,
            event.target.clone(),
            payload,
        );
        self.local_events.lock().unwrap().append(stored)?;
        Ok(())
    }

    pub fn publish(
        &self,
        source: CellId,
        target: CapabilityId,
        payload: EventPayload,
    ) -> Result<EventId> {
        let eid = self.next_event_id(source);
        let event = Event::new(
            eid,
            EventKind::Event,
            ProtocolRef::new("pantheon.eventbus.v1"),
            source,
            target,
            payload,
        );
        self.dispatch(event)
    }

    pub fn publish_command(
        &self,
        source: CellId,
        target: CapabilityId,
        payload: EventPayload,
    ) -> Result<EventId> {
        let eid = self.next_event_id(source);
        let event = Event::new(
            eid,
            EventKind::Command,
            ProtocolRef::new("pantheon.eventbus.v1"),
            source,
            target,
            payload,
        );
        self.dispatch(event)
    }

    pub fn dispatch(&self, event: Event) -> Result<EventId> {
        let event_id = event.id;
        self.store_event(&event)?;

        let targets: Vec<(String, u64)> = {
            let subs = self.subscriptions.read().unwrap();
            subs.iter()
                .filter(|s| s.matches(&event))
                .map(|s| (s.subscriber_id().to_string(), s.id()))
                .collect()
        };

        for (sub_id, _sub_id_val) in &targets {
            let subs = self.subscriptions.read().unwrap();
            if let Some(s) = subs.iter().find(|s| s.subscriber_id() == sub_id) {
                if let Err(e) = s.deliver(&event) {
                    eprintln!("Event delivery failed to {}: {}", sub_id, e);
                }
            }
        }

        Ok(event_id)
    }

    pub fn subscribe(
        &self,
        subscriber_id: impl Into<String>,
        capability_pattern: impl Into<String>,
        event_types: Vec<EventKind>,
        callback: SubscriberCallback,
    ) -> u64 {
        let id = self.next_sub_id.fetch_add(1, Ordering::SeqCst);
        let sub = Subscription::new(id, subscriber_id, capability_pattern, event_types, callback);
        self.subscriptions.write().unwrap().push(sub);
        id
    }

    pub fn unsubscribe(&self, subscription_id: u64) -> Result<()> {
        let mut subs = self.subscriptions.write().unwrap();
        let pos = subs.iter().position(|s| s.id() == subscription_id);
        match pos {
            Some(p) => {
                subs[p].deactivate();
                subs.remove(p);
                Ok(())
            }
            None => Err(Error::Custom(format!(
                "Subscription not found: {}",
                subscription_id
            ))),
        }
    }

    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().unwrap().len()
    }

    pub fn local_log(&self) -> &Arc<Mutex<EventLog>> {
        &self.local_events
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_and_subscribe() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));

        let r = received.clone();
        bus.subscribe(
            "test-subscriber",
            "test.cap",
            vec![EventKind::Event],
            Box::new(move |event| {
                r.lock().unwrap().push(event.id);
                Ok(())
            }),
        );

        let source = CellId::random();
        let eid = bus
            .publish(
                source,
                CapabilityId::new("test.cap"),
                EventPayload::new(vec![1, 2, 3], "application/octet-stream"),
            )
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));

        let delivered = received.lock().unwrap();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0], eid);
    }

    #[test]
    fn test_publish_command() {
        let bus = EventBus::new();
        let source = CellId::random();
        bus.publish_command(
                source,
                CapabilityId::new("cmd.test"),
                EventPayload::new(vec![], "text/plain"),
            )
            .unwrap();

        assert!(bus.subscription_count() == 0);
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU64::new(0));

        for i in 0..3 {
            let c = count.clone();
            bus.subscribe(
                format!("sub-{}", i),
                "test.cap",
                vec![EventKind::Event],
                Box::new(move |_event| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            );
        }

        let source = CellId::random();
        bus.publish(
            source,
            CapabilityId::new("test.cap"),
            EventPayload::new(vec![], "text/plain"),
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_subscription_filtering() {
        let bus = EventBus::new();
        let event_count = Arc::new(AtomicU64::new(0));
        let cmd_count = Arc::new(AtomicU64::new(0));

        {
            let c = event_count.clone();
            bus.subscribe(
                "event-listener",
                "test.cap",
                vec![EventKind::Event],
                Box::new(move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            );
        }
        {
            let c = cmd_count.clone();
            bus.subscribe(
                "cmd-listener",
                "test.cap",
                vec![EventKind::Command],
                Box::new(move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            );
        }

        let source = CellId::random();
        bus.publish(
            source,
            CapabilityId::new("test.cap"),
            EventPayload::new(vec![], "text/plain"),
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(event_count.load(Ordering::SeqCst), 1);
        assert_eq!(cmd_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_unsubscribe() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU64::new(0));

        let id = {
            let c = count.clone();
            bus.subscribe(
                "temp-sub",
                "test.cap",
                vec![EventKind::Event],
                Box::new(move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            )
        };

        bus.unsubscribe(id).unwrap();

        let source = CellId::random();
        bus.publish(
            source,
            CapabilityId::new("test.cap"),
            EventPayload::new(vec![], "text/plain"),
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_dispatch_called_only_once() {
        let bus = EventBus::new();
        let call_count = Arc::new(AtomicU64::new(0));

        {
            let c = call_count.clone();
            bus.subscribe(
                "counter",
                "test.cap",
                vec![EventKind::Event],
                Box::new(move |_| {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            );
        }

        let source = CellId::random();
        for _ in 0..5 {
            bus.publish(
                source,
                CapabilityId::new("test.cap"),
                EventPayload::new(vec![], "text/plain"),
            )
            .unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(call_count.load(Ordering::SeqCst), 5);
    }
}
