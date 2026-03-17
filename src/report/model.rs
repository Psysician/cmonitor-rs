use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domain::{LimitEvent, SessionBlock, UsageEntry};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReportTotals {
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub total_messages: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelStats {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveSessionReport {
    pub block_id: String,
    pub started_at: OffsetDateTime,
    pub ends_at: OffsetDateTime,
    pub totals: ReportTotals,
    pub warnings: Vec<LimitEvent>,
    pub per_model: Vec<ModelStats>,
    pub models: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportState {
    pub generated_at: OffsetDateTime,
    pub blocks: Vec<SessionBlock>,
    pub limits: Vec<LimitEvent>,
    pub totals: ReportTotals,
    pub active_session: Option<ActiveSessionReport>,
    pub custom_limit: Option<u64>,
}

pub fn aggregate_per_model(entries: &[UsageEntry]) -> Vec<ModelStats> {
    let mut map: BTreeMap<String, ModelStats> = BTreeMap::new();
    for entry in entries {
        let stats = map.entry(entry.model.clone()).or_insert_with(|| ModelStats {
            model: entry.model.clone(),
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            total_tokens: 0,
            cost_usd: 0.0,
        });
        stats.input_tokens += entry.tokens.input_tokens;
        stats.output_tokens += entry.tokens.output_tokens;
        stats.cache_creation_tokens += entry.tokens.cache_creation_tokens;
        stats.cache_read_tokens += entry.tokens.cache_read_tokens;
        stats.total_tokens += entry.tokens.total_tokens();
        stats.cost_usd += entry.cost_usd.unwrap_or(0.0);
    }
    let mut result: Vec<ModelStats> = map.into_values().collect();
    result.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
    result
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
                    per_model: aggregate_per_model(&block.entries),
                    models: block.models.clone(),
                });

        Self {
            generated_at,
            blocks,
            limits,
            totals,
            active_session,
            custom_limit: None,
        }
    }
}
