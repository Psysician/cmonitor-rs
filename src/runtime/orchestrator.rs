use std::collections::BTreeSet;

use time::OffsetDateTime;

use crate::analysis::{
    calculate_custom_limit, detect_limit_events_from_candidates, transform_to_blocks,
};
use crate::config::ResolvedConfig;
use crate::discovery::{collect_jsonl_files, discover_roots, select_primary_root};
use crate::parser::{parse_jsonl_file, DedupKey};
use crate::report::ReportState;

pub fn load_report_state(_resolved: &ResolvedConfig) -> anyhow::Result<ReportState> {
    let now = OffsetDateTime::now_utc();
    let discovery = discover_roots();
    let Some(root) = select_primary_root(&discovery) else {
        return Ok(ReportState::from_blocks(now, Vec::new(), Vec::new()));
    };

    let files = collect_jsonl_files(root);
    let mut all_entries = Vec::new();
    let mut all_limit_candidates = Vec::new();
    let mut seen: BTreeSet<DedupKey> = BTreeSet::new();

    for file in &files {
        let Ok(parsed) = parse_jsonl_file(file, &mut seen) else {
            continue;
        };
        all_entries.extend(parsed.entries);
        all_limit_candidates.extend(parsed.limit_candidates);
    }

    all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut blocks = transform_to_blocks(&all_entries, now);
    let limits = detect_limit_events_from_candidates(&all_limit_candidates, &mut blocks);
    let custom_limit = calculate_custom_limit(&blocks);
    let mut state = ReportState::from_blocks(now, blocks, limits);
    state.custom_limit = custom_limit;
    Ok(state)
}
