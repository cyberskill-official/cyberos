//! Background WAL consumer — in-memory receipts; cash-app stubbed until TASK-INV-006.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tracing::warn;

use super::types::{WiseEventType, WiseReceiptState};
use super::wal::{WiseWalItem, WiseWalQueue};

/// Documented stub — INV-006 cash-application cascade is not invoked in host-c.
#[derive(Debug, Default, Clone, Copy)]
pub struct CashAppStub;

impl CashAppStub {
    /// No-op until TASK-INV-006. Returns without matching invoices.
    pub fn apply(&self, _item: &WiseWalItem) {
        // Intentionally empty — see TASK-INV-012 AC #7.
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryReceipt {
    pub profile_id: i64,
    pub event_id: String,
    pub event_type: WiseEventType,
    pub state: WiseReceiptState,
}

#[derive(Debug)]
pub struct WiseProcessor {
    wal: Arc<Mutex<WiseWalQueue>>,
    seen: Mutex<HashSet<(i64, String)>>,
    receipts: Mutex<Vec<InMemoryReceipt>>,
    dead_letters: Mutex<Vec<(i64, String, &'static str)>>,
    cash_app: CashAppStub,
}

impl WiseProcessor {
    pub fn new(wal: Arc<Mutex<WiseWalQueue>>) -> Self {
        Self {
            wal,
            seen: Mutex::new(HashSet::new()),
            receipts: Mutex::new(Vec::new()),
            dead_letters: Mutex::new(Vec::new()),
            cash_app: CashAppStub,
        }
    }

    pub fn receipt_count(&self) -> usize {
        self.receipts.lock().map(|r| r.len()).unwrap_or(0)
    }

    pub fn has_receipt(&self, profile_id: i64, event_id: &str) -> bool {
        self.seen
            .lock()
            .map(|s| s.contains(&(profile_id, event_id.to_string())))
            .unwrap_or(false)
    }

    pub fn dead_letter_count(&self) -> usize {
        self.dead_letters.lock().map(|d| d.len()).unwrap_or(0)
    }

    /// Drain one WAL item if present. Idempotent on (profile_id, event_id).
    pub fn process_one(&self) -> bool {
        let item = match self.wal.lock() {
            Ok(mut q) => q.pop(),
            Err(e) => {
                warn!(error = %e, "wise wal lock poisoned");
                None
            }
        };
        let Some(item) = item else {
            return false;
        };
        self.accept(item);
        true
    }

    pub fn drain_all(&self) {
        while self.process_one() {}
    }

    fn accept(&self, item: WiseWalItem) {
        let key = (item.profile_id, item.event_id.clone());
        {
            let mut seen = self.seen.lock().expect("seen lock");
            if !seen.insert(key.clone()) {
                return;
            }
        }
        // Cash-app stub — no INV-006 cascade.
        self.cash_app.apply(&item);
        let mut receipts = self.receipts.lock().expect("receipts lock");
        receipts.push(InMemoryReceipt {
            profile_id: item.profile_id,
            event_id: item.event_id,
            event_type: item.event_type,
            state: WiseReceiptState::Received,
        });
    }

    pub fn dead_letter(&self, profile_id: i64, event_id: String, reason: &'static str) {
        if let Ok(mut dl) = self.dead_letters.lock() {
            dl.push((profile_id, event_id, reason));
        }
    }
}
