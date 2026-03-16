use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domain::{TokenUsage, UsageEntry};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LimitKind {
    Usage,
    Rate,
    Opus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LimitEvent {
    pub kind: LimitKind,
    pub timestamp: OffsetDateTime,
    pub message: String,
    pub reset_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionBlock {
    pub id: String,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub actual_end_time: Option<OffsetDateTime>,
    pub is_gap: bool,
    pub is_active: bool,
    pub entries: Vec<UsageEntry>,
    pub limits: Vec<LimitEvent>,
    pub tokens: TokenUsage,
    pub cost_usd: f64,
    pub message_count: usize,
    pub models: Vec<String>,
}

impl SessionBlock {
    pub fn empty_gap(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
        Self {
            id: format!("gap-{}", start_time.unix_timestamp()),
            start_time,
            end_time,
            actual_end_time: None,
            is_gap: true,
            is_active: false,
            entries: Vec::new(),
            limits: Vec::new(),
            tokens: TokenUsage::default(),
            cost_usd: 0.0,
            message_count: 0,
            models: Vec::new(),
        }
    }
}
