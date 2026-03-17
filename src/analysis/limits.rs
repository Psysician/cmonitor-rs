use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::domain::{LimitEvent, LimitKind, SessionBlock};
use crate::parser::{LimitCandidate, RawUsageEvent};

pub fn detect_limit_events(
    raw_events: &[RawUsageEvent],
    blocks: &mut [SessionBlock],
) -> Vec<LimitEvent> {
    let mut detected = Vec::new();

    for event in raw_events {
        let Some(limit) = parse_limit_event(event) else {
            continue;
        };

        if let Some(block) = blocks.iter_mut().find(|block| {
            !block.is_gap
                && limit.timestamp >= block.start_time
                && limit.timestamp <= block.end_time
        }) {
            block.limits.push(limit.clone());
        }
        detected.push(limit);
    }

    detected
}

fn parse_limit_event(event: &RawUsageEvent) -> Option<LimitEvent> {
    let entry_type = event.payload.get("type").and_then(Value::as_str)?;
    if !matches!(entry_type, "system" | "tool_result") {
        return None;
    }

    let content = limit_message_content(&event.payload)?;
    let lowered = content.to_lowercase();
    if !lowered.contains("limit") && !lowered.contains("rate") {
        return None;
    }

    let timestamp = event
        .payload
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok())?;

    let kind = if lowered.contains("opus") {
        LimitKind::Opus
    } else if lowered.contains("rate") {
        LimitKind::Rate
    } else {
        LimitKind::Usage
    };

    Some(LimitEvent {
        kind,
        timestamp,
        message: content,
        reset_at: None,
    })
}

pub fn detect_limit_events_from_candidates(
    candidates: &[LimitCandidate],
    blocks: &mut [SessionBlock],
) -> Vec<LimitEvent> {
    let mut detected = Vec::new();

    for candidate in candidates {
        if !matches!(candidate.entry_type.as_str(), "system" | "tool_result") {
            continue;
        }

        let Some(content) = limit_message_content_from_value(candidate.content.as_ref()) else {
            continue;
        };
        let lowered = content.to_lowercase();
        if !lowered.contains("limit") && !lowered.contains("rate") {
            continue;
        }

        let timestamp = candidate
            .timestamp
            .as_deref()
            .and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok());
        let Some(timestamp) = timestamp else {
            continue;
        };

        let kind = if lowered.contains("opus") {
            LimitKind::Opus
        } else if lowered.contains("rate") {
            LimitKind::Rate
        } else {
            LimitKind::Usage
        };

        let limit = LimitEvent {
            kind,
            timestamp,
            message: content,
            reset_at: None,
        };

        if let Some(block) = blocks.iter_mut().find(|block| {
            !block.is_gap
                && limit.timestamp >= block.start_time
                && limit.timestamp <= block.end_time
        }) {
            block.limits.push(limit.clone());
        }
        detected.push(limit);
    }

    detected
}

fn limit_message_content_from_value(content: Option<&Value>) -> Option<String> {
    let content = content?;
    if let Some(s) = content.as_str() {
        return Some(s.to_owned());
    }
    let arr = content.as_array()?;
    let parts = arr
        .iter()
        .filter_map(|item| {
            item.get("text")
                .and_then(Value::as_str)
                .or_else(|| item.as_str())
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// Accepts both plain-string and structured content payloads because upstream
/// limit warnings do not arrive under a single message schema. (ref: DL-002)
fn limit_message_content(payload: &Value) -> Option<String> {
    if let Some(content) = payload.get("content").and_then(Value::as_str) {
        return Some(content.to_owned());
    }

    let content = payload.get("content").and_then(Value::as_array)?;
    let parts = content
        .iter()
        .filter_map(|item| {
            item.get("text")
                .and_then(Value::as_str)
                .or_else(|| item.as_str())
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}
