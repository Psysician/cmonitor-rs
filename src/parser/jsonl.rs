use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::discovery::JsonlFile;
use crate::domain::pricing::calculate_entry_cost;
use crate::domain::{TokenUsage, UsageEntry};

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DedupKey {
    pub message_id: String,
    pub request_id: String,
}

// ---------------------------------------------------------------------------
// Legacy types — kept for backward compatibility with tests
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RawUsageEvent {
    pub source_file: PathBuf,
    pub line_number: usize,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonlDiagnostic {
    pub source_file: PathBuf,
    pub line_number: usize,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DecodedJsonl {
    pub events: Vec<RawUsageEvent>,
    pub diagnostics: Vec<JsonlDiagnostic>,
}

pub fn decode_jsonl_file(file: &JsonlFile) -> anyhow::Result<DecodedJsonl> {
    let reader = BufReader::new(File::open(&file.path)?);
    let mut decoded = DecodedJsonl::default();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(trimmed) {
            Ok(payload) => decoded.events.push(RawUsageEvent {
                source_file: file.path.clone(),
                line_number,
                payload,
            }),
            Err(error) => decoded.diagnostics.push(JsonlDiagnostic {
                source_file: file.path.clone(),
                line_number,
                message: error.to_string(),
            }),
        }
    }

    Ok(decoded)
}

// ---------------------------------------------------------------------------
// Optimised typed path
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawLine {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default, rename = "type")]
    entry_type: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    message_id: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default, rename = "requestId")]
    request_id_alt: Option<String>,
    #[serde(default)]
    usage: Option<UsagePayload>,
    #[serde(default)]
    message: Option<MessagePayload>,
    #[serde(default)]
    cost: Option<f64>,
    #[serde(default)]
    content: Option<Value>,
}

#[derive(Deserialize)]
struct UsagePayload {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
    #[serde(default)]
    cache_read_tokens: u64,
}

#[derive(Deserialize)]
struct MessagePayload {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<UsagePayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LimitCandidate {
    pub source_file: PathBuf,
    pub line_number: usize,
    pub timestamp: Option<String>,
    pub entry_type: String,
    pub content: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedFile {
    pub entries: Vec<UsageEntry>,
    pub limit_candidates: Vec<LimitCandidate>,
    pub diagnostics: Vec<JsonlDiagnostic>,
}

pub fn parse_jsonl_file(
    file: &JsonlFile,
    seen: &mut BTreeSet<DedupKey>,
) -> anyhow::Result<ParsedFile> {
    let reader = BufReader::new(File::open(&file.path)?);
    let mut out = ParsedFile::default();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let raw: RawLine = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(error) => {
                out.diagnostics.push(JsonlDiagnostic {
                    source_file: file.path.clone(),
                    line_number,
                    message: error.to_string(),
                });
                continue;
            }
        };

        let entry_type = raw.entry_type.as_deref().unwrap_or("").to_owned();

        if matches!(entry_type.as_str(), "system" | "tool_result") {
            out.limit_candidates.push(LimitCandidate {
                source_file: file.path.clone(),
                line_number,
                timestamp: raw.timestamp,
                entry_type,
                content: raw.content,
            });
            continue;
        }

        let timestamp = match raw
            .timestamp
            .as_deref()
            .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
        {
            Some(ts) => ts,
            None => continue,
        };

        let tokens = extract_tokens_from_raw(&raw, &entry_type);
        if tokens.total_tokens() == 0 {
            continue;
        }

        let message_id = raw
            .message_id
            .clone()
            .or_else(|| raw.message.as_ref().and_then(|m| m.id.clone()));
        let request_id = raw
            .request_id
            .clone()
            .or_else(|| raw.request_id_alt.clone());

        if let (Some(mid), Some(rid)) = (&message_id, &request_id) {
            let key = DedupKey {
                message_id: mid.clone(),
                request_id: rid.clone(),
            };
            if !seen.insert(key) {
                continue;
            }
        }

        let model = raw
            .model
            .as_deref()
            .or_else(|| raw.message.as_ref().and_then(|m| m.model.as_deref()))
            .unwrap_or("unknown")
            .to_lowercase();

        let cost_usd = raw
            .cost
            .or_else(|| Some(calculate_entry_cost(&model, &tokens)));

        out.entries.push(UsageEntry {
            timestamp,
            model,
            message_id,
            request_id,
            tokens,
            cost_usd,
            source_file: file.path.clone(),
            line_number,
        });
    }

    Ok(out)
}

fn extract_tokens_from_raw(raw: &RawLine, entry_type: &str) -> TokenUsage {
    let is_assistant = entry_type == "assistant";

    let message_usage = raw.message.as_ref().and_then(|m| m.usage.as_ref());
    let root_usage = raw.usage.as_ref();

    let sources: [Option<&UsagePayload>; 2] = if is_assistant {
        [message_usage, root_usage]
    } else {
        [root_usage, message_usage]
    };

    for source in sources.into_iter().flatten() {
        let input = source.input_tokens;
        let output = source.output_tokens;
        if input > 0 || output > 0 {
            return TokenUsage {
                input_tokens: input,
                output_tokens: output,
                cache_creation_tokens: source
                    .cache_creation_tokens
                    .max(source.cache_creation_input_tokens),
                cache_read_tokens: source.cache_read_input_tokens.max(source.cache_read_tokens),
            };
        }
    }

    TokenUsage::default()
}

// ---------------------------------------------------------------------------
// Parallel parsing
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct ParallelParseResult {
    pub entries: Vec<UsageEntry>,
    pub limit_candidates: Vec<LimitCandidate>,
    pub diagnostics: Vec<JsonlDiagnostic>,
}

pub fn parse_jsonl_files_parallel(
    files: &[JsonlFile],
    seen: &mut BTreeSet<DedupKey>,
) -> ParallelParseResult {
    let per_file: Vec<ParsedFile> = files
        .par_iter()
        .filter_map(|file| parse_jsonl_file_no_dedup(file).ok())
        .collect();

    let mut result = ParallelParseResult::default();

    for pf in per_file {
        result.limit_candidates.extend(pf.limit_candidates);
        result.diagnostics.extend(pf.diagnostics);

        for entry in pf.entries {
            if let (Some(mid), Some(rid)) = (&entry.message_id, &entry.request_id) {
                let key = DedupKey {
                    message_id: mid.clone(),
                    request_id: rid.clone(),
                };
                if !seen.insert(key) {
                    continue;
                }
            }
            result.entries.push(entry);
        }
    }

    result
}

pub fn parse_jsonl_file_no_dedup(file: &JsonlFile) -> anyhow::Result<ParsedFile> {
    let reader = BufReader::new(File::open(&file.path)?);
    let mut out = ParsedFile::default();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let raw: RawLine = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(error) => {
                out.diagnostics.push(JsonlDiagnostic {
                    source_file: file.path.clone(),
                    line_number,
                    message: error.to_string(),
                });
                continue;
            }
        };

        let entry_type = raw.entry_type.as_deref().unwrap_or("").to_owned();

        if matches!(entry_type.as_str(), "system" | "tool_result") {
            out.limit_candidates.push(LimitCandidate {
                source_file: file.path.clone(),
                line_number,
                timestamp: raw.timestamp,
                entry_type,
                content: raw.content,
            });
            continue;
        }

        let timestamp = match raw
            .timestamp
            .as_deref()
            .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
        {
            Some(ts) => ts,
            None => continue,
        };

        let tokens = extract_tokens_from_raw(&raw, &entry_type);
        if tokens.total_tokens() == 0 {
            continue;
        }

        let message_id = raw
            .message_id
            .clone()
            .or_else(|| raw.message.as_ref().and_then(|m| m.id.clone()));
        let request_id = raw
            .request_id
            .clone()
            .or_else(|| raw.request_id_alt.clone());

        let model = raw
            .model
            .as_deref()
            .or_else(|| raw.message.as_ref().and_then(|m| m.model.as_deref()))
            .unwrap_or("unknown")
            .to_lowercase();

        let cost_usd = raw
            .cost
            .or_else(|| Some(calculate_entry_cost(&model, &tokens)));

        out.entries.push(UsageEntry {
            timestamp,
            model,
            message_id,
            request_id,
            tokens,
            cost_usd,
            source_file: file.path.clone(),
            line_number,
        });
    }

    Ok(out)
}
