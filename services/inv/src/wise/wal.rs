//! Bounded in-memory WAL for Wise webhook offload (DEC-850).

use std::collections::VecDeque;

use super::types::WiseEventType;

pub const WISE_WAL_CAPACITY: usize = 10_000;
pub const WISE_WAL_BACKPRESSURE_PCT: usize = 90;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiseWalItem {
    pub profile_id: i64,
    pub event_id: String,
    pub event_type: WiseEventType,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WiseWalError {
    #[error("wise_wal_overflow")]
    Overflow,
}

#[derive(Debug)]
pub struct WiseWalQueue {
    buf: VecDeque<WiseWalItem>,
    capacity: usize,
}

impl Default for WiseWalQueue {
    fn default() -> Self {
        Self::new(WISE_WAL_CAPACITY)
    }
}

impl WiseWalQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(capacity.min(256)),
            capacity,
        }
    }

    pub fn depth(&self) -> usize {
        self.buf.len()
    }

    pub fn push(&mut self, item: WiseWalItem) -> Result<(), WiseWalError> {
        if self.buf.len() * 100 >= self.capacity * WISE_WAL_BACKPRESSURE_PCT {
            return Err(WiseWalError::Overflow);
        }
        if self.buf.len() >= self.capacity {
            return Err(WiseWalError::Overflow);
        }
        self.buf.push_back(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<WiseWalItem> {
        self.buf.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backpressure_at_90_percent() {
        let mut q = WiseWalQueue::new(10);
        for i in 0..9 {
            q.push(WiseWalItem {
                profile_id: 1,
                event_id: format!("e{i}"),
                event_type: WiseEventType::BalancesCredit,
                body: vec![],
            })
            .unwrap();
        }
        assert_eq!(
            q.push(WiseWalItem {
                profile_id: 1,
                event_id: "overflow".into(),
                event_type: WiseEventType::BalancesCredit,
                body: vec![],
            }),
            Err(WiseWalError::Overflow)
        );
    }
}
