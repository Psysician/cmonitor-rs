use chrono::{DateTime, Duration as ChronoDuration, LocalResult, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
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
    let rate_limit_error = is_rate_limit_payload(&event.payload);
    if entry_type == "assistant" && !rate_limit_error {
        return None;
    }
    if !matches!(entry_type, "system" | "tool_result" | "assistant") && !rate_limit_error {
        return None;
    }

    let content = limit_message_content(&event.payload)
        .or_else(|| rate_limit_error.then(|| "Rate limit reached".to_owned()))?;
    let lowered = content.to_lowercase();
    if !looks_like_limit_message(&lowered, rate_limit_error) {
        return None;
    }

    let timestamp = event
        .payload
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok())?;

    let kind = classify_limit_kind(&lowered, rate_limit_error);

    let reset_at = parse_reset_at(&content, timestamp);

    Some(LimitEvent {
        kind,
        timestamp,
        message: content,
        reset_at,
    })
}

pub fn detect_limit_events_from_candidates(
    candidates: &[LimitCandidate],
    blocks: &mut [SessionBlock],
) -> Vec<LimitEvent> {
    let mut detected = Vec::new();

    for candidate in candidates {
        let rate_limit_error = is_rate_limit_candidate(candidate);
        if candidate.entry_type == "assistant" && !rate_limit_error {
            continue;
        }
        if !matches!(
            candidate.entry_type.as_str(),
            "system" | "tool_result" | "assistant"
        ) && !rate_limit_error
        {
            continue;
        }

        let Some(content) = limit_message_content_from_value(candidate.content.as_ref())
            .or_else(|| rate_limit_error.then(|| "Rate limit reached".to_owned()))
        else {
            continue;
        };
        let lowered = content.to_lowercase();
        if !looks_like_limit_message(&lowered, rate_limit_error) {
            continue;
        }

        let timestamp = candidate
            .timestamp
            .as_deref()
            .and_then(|raw| OffsetDateTime::parse(raw, &Rfc3339).ok());
        let Some(timestamp) = timestamp else {
            continue;
        };

        let reset_at = parse_reset_at(&content, timestamp);
        let limit = LimitEvent {
            kind: classify_limit_kind(&lowered, rate_limit_error),
            timestamp,
            message: content,
            reset_at,
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

pub fn mark_limited_blocks_active(blocks: &mut [SessionBlock], now: OffsetDateTime) {
    for block in blocks {
        if block.is_gap {
            continue;
        }

        if block
            .limits
            .iter()
            .filter_map(|limit| limit.reset_at)
            .any(|reset_at| reset_at > now)
        {
            block.is_active = true;
        }
    }
}

fn limit_message_content_from_value(content: Option<&Value>) -> Option<String> {
    let content = content?;
    let mut parts = Vec::new();
    collect_text_parts(content, &mut parts);
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// Accepts both plain-string and structured content payloads because upstream
/// limit warnings do not arrive under a single message schema. (ref: DL-002)
fn limit_message_content(payload: &Value) -> Option<String> {
    if let Some(content) = limit_message_content_from_value(payload.get("content")) {
        return Some(content);
    }

    limit_message_content_from_value(payload.get("message").and_then(|message| {
        message.get("content").or_else(|| {
            message
                .get("message")
                .and_then(|inner| inner.get("content"))
        })
    }))
}

fn collect_text_parts(value: &Value, parts: &mut Vec<String>) {
    if let Some(text) = value.as_str() {
        parts.push(text.to_owned());
        return;
    }

    if let Some(items) = value.as_array() {
        for item in items {
            collect_text_parts(item, parts);
        }
        return;
    }

    if let Some(text) = value.get("text").and_then(Value::as_str) {
        parts.push(text.to_owned());
    }
    if let Some(content) = value.get("content") {
        collect_text_parts(content, parts);
    }
}

fn is_rate_limit_payload(payload: &Value) -> bool {
    payload.get("error").and_then(Value::as_str) == Some("rate_limit")
        || payload.get("apiErrorStatus").and_then(Value::as_u64) == Some(429)
}

fn is_rate_limit_candidate(candidate: &LimitCandidate) -> bool {
    candidate.error.as_deref() == Some("rate_limit") || candidate.api_error_status == Some(429)
}

fn classify_limit_kind(lowered: &str, rate_limit_error: bool) -> LimitKind {
    if lowered.contains("opus") {
        LimitKind::Opus
    } else if rate_limit_error
        || lowered.contains("rate limit")
        || lowered.contains("too many requests")
    {
        LimitKind::Rate
    } else {
        LimitKind::Usage
    }
}

fn looks_like_limit_message(lowered: &str, rate_limit_error: bool) -> bool {
    rate_limit_error
        || contains_phrase_boundary(lowered, "rate limit")
        || contains_phrase_boundary(lowered, "rate-limit")
        || contains_phrase_boundary(lowered, "usage limit")
        || contains_phrase_boundary(lowered, "hit your limit")
        || contains_phrase_boundary(lowered, "limit reached")
        || contains_phrase_boundary(lowered, "limit exceeded")
        || contains_phrase_boundary(lowered, "limit hit")
        || contains_phrase_boundary(lowered, "limit nearly exhausted")
        || lowered.contains("too many requests")
        || (lowered.contains("resets") && lowered.contains("limit"))
}

fn contains_phrase_boundary(haystack: &str, phrase: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut start_at = 0usize;
    while let Some(found) = haystack[start_at..].find(phrase) {
        let start = start_at + found;
        let end = start + phrase.len();
        let before_is_word = start
            .checked_sub(1)
            .and_then(|idx| bytes.get(idx))
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
        let after_is_word = bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
        if !before_is_word && !after_is_word {
            return true;
        }
        start_at = start + 1;
    }
    false
}

fn parse_reset_at(content: &str, timestamp: OffsetDateTime) -> Option<OffsetDateTime> {
    parse_pipe_reset_timestamp(content)
        .or_else(|| parse_wait_minutes_reset(content, timestamp))
        .or_else(|| parse_clock_reset(content, timestamp))
}

fn parse_pipe_reset_timestamp(content: &str) -> Option<OffsetDateTime> {
    let lowered = content.to_ascii_lowercase();
    let marker = "limit reached|";
    let idx = lowered.find(marker)?;
    let after = &content[idx + marker.len()..];
    let digits = take_ascii_digits(after)?;
    let ts = digits.parse::<i64>().ok()?;
    OffsetDateTime::from_unix_timestamp(ts).ok()
}

fn parse_wait_minutes_reset(content: &str, timestamp: OffsetDateTime) -> Option<OffsetDateTime> {
    let lowered = content.to_ascii_lowercase();
    let idx = lowered.find("wait")?;
    let after = lowered[idx + "wait".len()..].trim_start();
    let digits = take_ascii_digits(after)?;
    let minutes = digits.parse::<i64>().ok()?;
    timestamp.checked_add(time::Duration::minutes(minutes))
}

fn parse_clock_reset(content: &str, timestamp: OffsetDateTime) -> Option<OffsetDateTime> {
    let lowered = content.to_ascii_lowercase();
    let marker = ["resets at", "resets", "reset at", "reset"]
        .into_iter()
        .find_map(|marker| lowered.find(marker).map(|idx| (idx, marker)))?;
    let after = &content[marker.0 + marker.1.len()..];
    let (hour, minute) = parse_clock_time(after)?;
    let timezone = parse_reset_timezone(content).unwrap_or(chrono_tz::UTC);
    clock_reset_in_timezone(timestamp, timezone, hour, minute)
}

fn parse_clock_time(input: &str) -> Option<(u8, u8)> {
    let trimmed = input.trim_start_matches(|c: char| c.is_whitespace() || c == ':' || c == '-');
    let bytes = trimmed.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos == 0 {
        return None;
    }

    let mut hour = trimmed[..pos].parse::<u8>().ok()?;
    let mut minute = 0u8;

    if bytes.get(pos) == Some(&b':') {
        pos += 1;
        let minute_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos == minute_start {
            return None;
        }
        minute = trimmed[minute_start..pos].parse::<u8>().ok()?;
    }

    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }

    let suffix = trimmed[pos..].to_ascii_lowercase();
    if suffix.starts_with("am") {
        if !(1..=12).contains(&hour) {
            return None;
        }
        if hour == 12 {
            hour = 0;
        }
    } else if suffix.starts_with("pm") {
        if !(1..=12).contains(&hour) {
            return None;
        }
        if hour != 12 {
            hour += 12;
        }
    } else if hour > 23 {
        return None;
    }

    if minute > 59 {
        return None;
    }

    Some((hour, minute))
}

fn parse_reset_timezone(content: &str) -> Option<Tz> {
    let open = content.rfind('(')?;
    let close = content[open + 1..].find(')')? + open + 1;
    content[open + 1..close].trim().parse::<Tz>().ok()
}

fn clock_reset_in_timezone(
    timestamp: OffsetDateTime,
    timezone: Tz,
    hour: u8,
    minute: u8,
) -> Option<OffsetDateTime> {
    let event_utc =
        DateTime::<Utc>::from_timestamp(timestamp.unix_timestamp(), timestamp.nanosecond())?;
    let local_date = event_utc.with_timezone(&timezone).date_naive();
    let local_time = NaiveTime::from_hms_opt(hour.into(), minute.into(), 0)?;

    let mut reset_utc = resolve_local_reset(timezone, local_date, local_time)?;
    if reset_utc <= event_utc {
        let next_date = local_date.checked_add_signed(ChronoDuration::days(1))?;
        reset_utc = resolve_local_reset(timezone, next_date, local_time)?;
    }

    OffsetDateTime::from_unix_timestamp(reset_utc.timestamp()).ok()
}

fn resolve_local_reset(
    timezone: Tz,
    date: chrono::NaiveDate,
    time: NaiveTime,
) -> Option<DateTime<Utc>> {
    match timezone.from_local_datetime(&date.and_time(time)) {
        LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(first, second) => {
            let dt = if first <= second { first } else { second };
            Some(dt.with_timezone(&Utc))
        }
        LocalResult::None => None,
    }
}

fn take_ascii_digits(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        pos += 1;
    }
    (pos > 0).then_some(&input[..pos])
}
