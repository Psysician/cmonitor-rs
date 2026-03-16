use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::discovery::JsonlFile;

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
