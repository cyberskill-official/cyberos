use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use super::types::{WiseEventType, WISE_STALE_DAYS};

#[derive(Debug, Clone, Deserialize)]
pub struct WiseEvent {
    pub event_type: String,
    pub data: WiseEventData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WiseEventData {
    pub resource: WiseResource,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WiseResource {
    pub id: String,
    #[serde(default)]
    pub profile_id: Option<i64>,
}

pub fn parse_event_type(raw: &str) -> Option<WiseEventType> {
    match raw {
        "transfers#state-change" => Some(WiseEventType::TransfersStateChange),
        "balances#credit" => Some(WiseEventType::BalancesCredit),
        "balances#update" => Some(WiseEventType::BalancesUpdate),
        _ => None,
    }
}

pub fn is_stale(occurred_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now - occurred_at > Duration::days(WISE_STALE_DAYS)
}

pub fn profile_id_mismatch(url_profile_id: i64, body_profile_id: Option<i64>) -> bool {
    match body_profile_id {
        Some(b) => b != url_profile_id,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_types() {
        assert_eq!(
            parse_event_type("transfers#state-change"),
            Some(WiseEventType::TransfersStateChange)
        );
        assert!(parse_event_type("unknown").is_none());
    }

    #[test]
    fn stale_after_five_days() {
        let occurred = Utc::now() - Duration::days(6);
        assert!(is_stale(occurred, Utc::now()));
    }
}
