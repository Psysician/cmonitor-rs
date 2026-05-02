use time::{Duration, OffsetDateTime, UtcOffset};

use crate::domain::{SessionBlock, TokenUsage, UsageEntry};

pub const SESSION_WINDOW_MINUTES: i64 = 10;

pub fn transform_to_blocks(entries: &[UsageEntry], now: OffsetDateTime) -> Vec<SessionBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut current = new_block(&entries[0]);

    for entry in entries.iter().cloned() {
        let should_roll = entry.timestamp >= current.end_time
            || current.entries.last().is_some_and(|last| {
                entry.timestamp - last.timestamp >= Duration::minutes(SESSION_WINDOW_MINUTES)
            });

        if should_roll {
            finalize_block(&mut current, now);
            if let Some(last_end) = current.actual_end_time {
                blocks.push(current);
                if entry.timestamp - last_end >= Duration::minutes(SESSION_WINDOW_MINUTES) {
                    blocks.push(SessionBlock::empty_gap(last_end, entry.timestamp));
                }
                current = new_block(&entry);
            } else {
                blocks.push(current);
                current = new_block(&entry);
            }
        }

        append_entry(&mut current, entry);
    }

    finalize_block(&mut current, now);
    blocks.push(current);
    blocks
}

fn new_block(entry: &UsageEntry) -> SessionBlock {
    let start_time = round_down_to_10min(entry.timestamp);
    SessionBlock {
        id: start_time.unix_timestamp().to_string(),
        start_time,
        end_time: start_time + Duration::minutes(SESSION_WINDOW_MINUTES),
        actual_end_time: None,
        is_gap: false,
        is_active: false,
        entries: Vec::new(),
        limits: Vec::new(),
        tokens: TokenUsage::default(),
        cost_usd: 0.0,
        message_count: 0,
        models: Vec::new(),
        model_stats: Vec::new(),
    }
}

fn append_entry(block: &mut SessionBlock, entry: UsageEntry) {
    block.tokens.input_tokens += entry.tokens.input_tokens;
    block.tokens.output_tokens += entry.tokens.output_tokens;
    block.tokens.cache_creation_tokens += entry.tokens.cache_creation_tokens;
    block.tokens.cache_read_tokens += entry.tokens.cache_read_tokens;
    block.cost_usd += entry.cost_usd.unwrap_or_default();
    block.message_count += 1;
    if !block.models.iter().any(|model| model == &entry.model) {
        block.models.push(entry.model.clone());
    }
    block.entries.push(entry);
}

fn finalize_block(block: &mut SessionBlock, now: OffsetDateTime) {
    block.actual_end_time = block.entries.last().map(|entry| entry.timestamp);
    block.is_active = !block.is_gap && block.end_time > now;
}

fn round_down_to_10min(timestamp: OffsetDateTime) -> OffsetDateTime {
    let utc = timestamp.to_offset(UtcOffset::UTC);
    let minute_remainder = i64::from(utc.minute() % 10);
    let offset = minute_remainder * 60 + i64::from(utc.second());
    OffsetDateTime::from_unix_timestamp(utc.unix_timestamp() - offset)
        .expect("rounded timestamp should be valid")
}
