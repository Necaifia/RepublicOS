use crate::types::LamportTimestamp;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct LamportClock {
    counter: AtomicU64,
}

impl LamportClock {
    pub const fn new(initial: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial),
        }
    }

    pub fn tick(&self) -> LamportTimestamp {
        let prev = self.counter.fetch_add(1, Ordering::AcqRel);
        LamportTimestamp::new(prev + 1)
    }

    pub fn observe(&self, other: LamportTimestamp) -> LamportTimestamp {
        loop {
            let current = self.counter.load(Ordering::Acquire);
            let new_value = current.max(other.value()) + 1;
            if self
                .counter
                .compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return LamportTimestamp::new(new_value);
            }
        }
    }

    pub fn peek(&self) -> LamportTimestamp {
        LamportTimestamp::new(self.counter.load(Ordering::Acquire))
    }

    pub fn reset(&self) {
        self.counter.store(0, Ordering::Release);
    }
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_increments() {
        let clock = LamportClock::new(0);
        let t1 = clock.tick();
        let t2 = clock.tick();
        assert_eq!(t1, LamportTimestamp::new(1));
        assert_eq!(t2, LamportTimestamp::new(2));
    }

    #[test]
    fn test_observe_advances() {
        let clock = LamportClock::new(0);
        let result = clock.observe(LamportTimestamp::new(10));
        assert_eq!(result, LamportTimestamp::new(11));
        assert_eq!(clock.peek(), LamportTimestamp::new(11));
    }

    #[test]
    fn test_observe_does_not_go_backward() {
        let clock = LamportClock::new(100);
        let result = clock.observe(LamportTimestamp::new(5));
        assert_eq!(result, LamportTimestamp::new(101));
        assert_eq!(clock.peek(), LamportTimestamp::new(101));
    }

    #[test]
    fn test_peek() {
        let clock = LamportClock::new(0);
        assert_eq!(clock.peek(), LamportTimestamp::new(0));
        clock.tick();
        assert_eq!(clock.peek(), LamportTimestamp::new(1));
    }

    #[test]
    fn test_concurrent_ticks() {
        let clock = std::sync::Arc::new(LamportClock::new(0));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let c = clock.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    c.tick();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(clock.peek(), LamportTimestamp::new(1000));
    }
}
