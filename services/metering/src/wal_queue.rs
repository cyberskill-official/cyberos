//! Bounded in-memory WAL queue (DEC-709) — 100k event ring; back-pressure at 90%.

use std::collections::VecDeque;

use crate::recorder::MeteringEvent;

pub const WAL_CAPACITY: usize = 100_000;
pub const WAL_BACKPRESSURE_PCT: usize = 90;

#[derive(Debug)]
pub struct WalQueue {
    buf: VecDeque<MeteringEvent>,
    capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WalError {
    #[error("wal_queue_overflow")]
    Overflow,
}

impl Default for WalQueue {
    fn default() -> Self {
        Self::new(WAL_CAPACITY)
    }
}

impl WalQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
        }
    }

    pub fn depth(&self) -> usize {
        self.buf.len()
    }

    pub fn push(&mut self, event: MeteringEvent) -> Result<(), WalError> {
        if self.buf.len() * 100 >= self.capacity * WAL_BACKPRESSURE_PCT {
            return Err(WalError::Overflow);
        }
        if self.buf.len() >= self.capacity {
            return Err(WalError::Overflow);
        }
        self.buf.push_back(event);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<MeteringEvent> {
        self.buf.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axes::MeteringAxis;
    use chrono::Utc;

    fn evt(key: &str) -> MeteringEvent {
        MeteringEvent {
            tenant_id: "t".into(),
            axis: MeteringAxis::ApiCalls,
            quantity: 1,
            idempotency_key: key.into(),
            source_service: "test".into(),
            occurred_at: Utc::now(),
            extra: serde_json::json!({}),
        }
    }

    #[test]
    fn backpressure_at_90_percent() {
        let mut q = WalQueue::new(10);
        for i in 0..9 {
            q.push(evt(&format!("k{i}"))).unwrap();
        }
        assert_eq!(q.push(evt("overflow")), Err(WalError::Overflow));
    }
}
