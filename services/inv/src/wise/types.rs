use serde::{Deserialize, Serialize};

pub const WISE_EVENT_TYPE_CARDINALITY: usize = 3;
pub const WISE_RECEIPT_STATE_CARDINALITY: usize = 5;
pub const WISE_STALE_DAYS: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WiseEventType {
    TransfersStateChange,
    BalancesCredit,
    BalancesUpdate,
}

impl WiseEventType {
    pub const ALL: [WiseEventType; WISE_EVENT_TYPE_CARDINALITY] = [
        WiseEventType::TransfersStateChange,
        WiseEventType::BalancesCredit,
        WiseEventType::BalancesUpdate,
    ];

    pub fn as_wire(self) -> &'static str {
        match self {
            WiseEventType::TransfersStateChange => "transfers#state-change",
            WiseEventType::BalancesCredit => "balances#credit",
            WiseEventType::BalancesUpdate => "balances#update",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WiseReceiptState {
    Received,
    Matched,
    CurrencyMismatch,
    DeadLettered,
    ManuallyResolved,
}

impl WiseReceiptState {
    pub const ALL: [WiseReceiptState; WISE_RECEIPT_STATE_CARDINALITY] = [
        WiseReceiptState::Received,
        WiseReceiptState::Matched,
        WiseReceiptState::CurrencyMismatch,
        WiseReceiptState::DeadLettered,
        WiseReceiptState::ManuallyResolved,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_cardinality() {
        assert_eq!(WiseEventType::ALL.len(), 3);
    }

    #[test]
    fn receipt_state_cardinality() {
        assert_eq!(WiseReceiptState::ALL.len(), 5);
    }
}
