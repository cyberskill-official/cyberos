//! Closed metering axes (DEC-704).

use serde::{Deserialize, Serialize};

pub const METERING_AXIS_CARDINALITY: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeteringAxis {
    Seats,
    ApiCalls,
    AiTokens,
    StorageBytes,
}

impl MeteringAxis {
    pub const ALL: [MeteringAxis; METERING_AXIS_CARDINALITY] = [
        MeteringAxis::Seats,
        MeteringAxis::ApiCalls,
        MeteringAxis::AiTokens,
        MeteringAxis::StorageBytes,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            MeteringAxis::Seats => "seats",
            MeteringAxis::ApiCalls => "api_calls",
            MeteringAxis::AiTokens => "ai_tokens",
            MeteringAxis::StorageBytes => "storage_bytes",
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            MeteringAxis::Seats => "seat",
            MeteringAxis::ApiCalls => "request",
            MeteringAxis::AiTokens => "token",
            MeteringAxis::StorageBytes => "byte",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinality_is_four() {
        assert_eq!(MeteringAxis::ALL.len(), 4);
    }
}
