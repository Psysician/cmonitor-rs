use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::domain::{TokenUsage, UsageEntry};
use crate::parser::jsonl::{DecodedJsonl, RawUsageEvent};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DedupKey {
    pub message_id: String,
    pub request_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntryNormalization {
    pub entries: Vec<UsageEntry>,
    pub retained_raw_events: Vec<RawUsageEvent>,
    pub preserved_with_tokens: usize,
    pub skipped_zero_tokens: usize,
    pub skipped_before_cutoff: usize,
    pub skipped_duplicates: usize,
}

pub fn normalize_usage_entries(
    decoded: DecodedJsonl,
    cutoff: Option<OffsetDateTime>,
) -> EntryNormalization {
    let mut accepted = Vec::new();
    let mut preserved_raw_events = Vec::new();
    let mut seen = BTreeSet::new();
    let mut report = EntryNormalization::default();

    for event in decoded.events {
        let Some(timestamp) = parse_timestamp(&event.payload) else {
            continue;
        };

        if cutoff.is_some_and(|limit| timestamp < limit) {
            report.skipped_before_cutoff += 1;
            continue;
        }

        // System and tool_result rows stay in the raw stream because limit
        // warnings can be meaningful even when token totals are zero. (ref: DL-002)
        if should_preserve_raw_event(&event.payload) {
            if has_nonzero_tokens(&event.payload) {
                report.preserved_with_tokens += 1;
            }
            preserved_raw_events.push(event.clone());
            continue;
        }

        let tokens = extract_tokens(&event.payload);
        if tokens.total_tokens() == 0 {
            report.skipped_zero_tokens += 1;
            continue;
        }

        if let Some(key) = dedup_key(&event.payload)
            && !seen.insert(key)
        {
            report.skipped_duplicates += 1;
            continue;
        }

        let entry = UsageEntry {
            timestamp,
            model: normalize_model(&event.payload),
            message_id: message_id(&event.payload),
            request_id: request_id(&event.payload),
            tokens,
            cost_usd: event.payload.get("cost").and_then(Value::as_f64),
            source_file: event.source_file.clone(),
            line_number: event.line_number,
        };
        accepted.push((entry, event));
    }

    accepted.sort_by(|left, right| left.0.timestamp.cmp(&right.0.timestamp));
    report.entries = accepted.iter().map(|(entry, _)| entry.clone()).collect();
    preserved_raw_events.extend(accepted.into_iter().map(|(_, event)| event));
    preserved_raw_events.sort_by_key(event_sort_key);
    report.retained_raw_events = preserved_raw_events;
    report
}

fn parse_timestamp(payload: &Value) -> Option<OffsetDateTime> {
    payload
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|value| OffsetDateTime::parse(value, &Rfc3339).ok())
}

fn extract_tokens(payload: &Value) -> TokenUsage {
    let usage = payload.get("usage").cloned().unwrap_or(Value::Null);
    TokenUsage {
        input_tokens: usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        output_tokens: usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_creation_tokens: usage
            .get("cache_creation_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        cache_read_tokens: usage
            .get("cache_read_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
    }
}

fn message_id(payload: &Value) -> Option<String> {
    payload
        .get("message_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            payload
                .get("message")
                .and_then(|message| message.get("id"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn request_id(payload: &Value) -> Option<String> {
    payload
        .get("request_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            payload
                .get("requestId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn dedup_key(payload: &Value) -> Option<DedupKey> {
    Some(DedupKey {
        message_id: message_id(payload)?,
        request_id: request_id(payload)?,
    })
}

fn normalize_model(payload: &Value) -> String {
    payload
        .get("model")
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("message")
                .and_then(|message| message.get("model"))
                .and_then(Value::as_str)
        })
        .unwrap_or("unknown")
        .to_lowercase()
}

/// Keeps warning-only rows available to limit detection while usage filtering
/// removes entries that do not contribute token totals. (ref: DL-002)
fn should_preserve_raw_event(payload: &Value) -> bool {
    matches!(
        payload.get("type").and_then(Value::as_str),
        Some("system" | "tool_result")
    )
}

fn has_nonzero_tokens(payload: &Value) -> bool {
    payload
        .get("usage")
        .and_then(|usage| {
            let input = usage
                .get("input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let output = usage
                .get("output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if input + output > 0 { Some(()) } else { None }
        })
        .is_some()
}

/// Sorts preserved raw rows deterministically; unparseable timestamps sort last
/// so they do not interfere with block attachment. (ref: DL-002)
fn event_sort_key(event: &RawUsageEvent) -> (i64, String, usize) {
    (
        parse_timestamp(&event.payload)
            .map(|ts| ts.unix_timestamp())
            .unwrap_or(i64::MAX),
        event.source_file.display().to_string(),
        event.line_number,
    )
}
