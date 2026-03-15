use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domain::{LimitEvent, SessionBlock};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReportTotals {
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub total_messages: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveSessionReport {
    pub block_id: String,
    pub started_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
    pub totals: ReportTotals,
    pub warnings: Vec<LimitEvent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportState {
    pub generated_at: OffsetDateTime,
    pub blocks: Vec<SessionBlock>,
    pub limits: Vec<LimitEvent>,
    pub totals: ReportTotals,
    pub active_session: Option<ActiveSessionReport>,
}

impl ReportState {
    pub fn from_blocks(
        generated_at: OffsetDateTime,
        blocks: Vec<SessionBlock>,
        limits: Vec<LimitEvent>,
    ) -> Self {
        let totals = blocks
            .iter()
            .fold(ReportTotals::default(), |mut totals, block| {
                totals.total_tokens += block.tokens.total_tokens();
                totals.total_cost_usd += block.cost_usd;
                totals.total_messages += block.message_count;
                totals
            });

        let active_session =
            blocks
                .iter()
                .find(|block| block.is_active)
                .map(|block| ActiveSessionReport {
                    block_id: block.id.clone(),
                    started_at: block.start_time,
                    ends_at: block.end_time,
                    totals: ReportTotals {
                        total_tokens: block.tokens.total_tokens(),
                        total_cost_usd: block.cost_usd,
                        total_messages: block.message_count,
                    },
                    warnings: block.limits.clone(),
                });

        Self {
            generated_at,
            blocks,
            limits,
            totals,
            active_session,
        }
    }
}
