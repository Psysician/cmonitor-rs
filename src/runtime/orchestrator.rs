use time::OffsetDateTime;

use crate::analysis::{calculate_custom_limit, detect_limit_events, transform_to_blocks};
use crate::config::ResolvedConfig;
use crate::discovery::{collect_jsonl_files, discover_roots, select_primary_root};
use crate::parser::{DecodedJsonl, EntryNormalization, decode_jsonl_file, normalize_usage_entries};
use crate::report::ReportState;

pub fn load_report_state(_resolved: &ResolvedConfig) -> anyhow::Result<ReportState> {
    let now = OffsetDateTime::now_utc();
    let discovery = discover_roots();
    let Some(root) = select_primary_root(&discovery) else {
        return Ok(ReportState::from_blocks(now, Vec::new(), Vec::new()));
    };

    let mut decoded = DecodedJsonl::default();
    for file in collect_jsonl_files(root) {
        let Ok(file_decoded) = decode_jsonl_file(&file) else {
            continue;
        };
        decoded.events.extend(file_decoded.events);
        decoded.diagnostics.extend(file_decoded.diagnostics);
    }

    // Usage entries drive block math, while preserved raw rows keep zero-token
    // warnings available for limit detection in the same load pass. (ref: DL-002)
    let EntryNormalization {
        entries,
        retained_raw_events,
        ..
    } = normalize_usage_entries(decoded, None);
    let mut blocks = transform_to_blocks(&entries, now);
    let limits = detect_limit_events(&retained_raw_events, &mut blocks);
    let _custom_limit = calculate_custom_limit(&blocks);
    Ok(ReportState::from_blocks(now, blocks, limits))
}
