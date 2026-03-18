use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use time::OffsetDateTime;

use crate::analysis::{
    calculate_custom_limit, detect_limit_events_from_candidates, transform_to_blocks,
};
use crate::config::ResolvedConfig;
use crate::discovery::{collect_jsonl_files, discover_roots, select_primary_root, select_roots};
use crate::parser::{DedupKey, ParsedFile, parse_jsonl_file, parse_jsonl_files_parallel};
use crate::report::ReportState;

/// Per-cycle parse cache for the realtime loop. Maps each file path to its
/// last-seen mtime and the `ParsedFile` produced at that mtime. Files whose
/// mtime is unchanged on the next cycle reuse the cached result, avoiding a
/// redundant read and parse. (ref: DL-002)
pub struct DeltaCache {
    entries: HashMap<PathBuf, (SystemTime, ParsedFile)>,
}

impl DeltaCache {
    /// Returns an empty cache. Call once before starting the realtime loop.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

/// Loads report state from JSONL files under the primary discovered root.
///
/// When `delta_cache` is `Some`, files with an unchanged mtime reuse their
/// cached `ParsedFile`; only new or modified files are re-parsed. Table mode
/// passes `None` because one-shot runs have no reuse opportunity. (ref: DL-002)
pub fn load_report_state(
    resolved: &ResolvedConfig,
    delta_cache: Option<&mut DeltaCache>,
) -> anyhow::Result<ReportState> {
    let now = OffsetDateTime::now_utc();
    let discovery = discover_roots();

    let roots: Vec<_> = if resolved.cli.multi_root {
        select_roots(&discovery)
    } else {
        select_primary_root(&discovery).into_iter().collect()
    };

    if roots.is_empty() {
        return Ok(ReportState::from_blocks(now, Vec::new(), Vec::new()));
    }

    let mut files = Vec::new();
    for root in roots {
        files.extend(collect_jsonl_files(root, resolved.cli.since_threshold));
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    let mut all_entries = Vec::new();
    let mut all_limit_candidates = Vec::new();
    let mut seen: BTreeSet<DedupKey> = BTreeSet::new();

    match delta_cache {
        Some(cache) => {
            let mut files_to_parse = Vec::new();
            for file in &files {
                let mtime = file.path.metadata().ok().and_then(|m| m.modified().ok());
                if let Some(mt) = mtime {
                    if let Some((cached_mt, pf)) = cache.entries.get(&file.path) {
                        if *cached_mt == mt {
                            all_entries.extend(pf.entries.clone());
                            all_limit_candidates.extend(pf.limit_candidates.clone());
                            continue;
                        }
                    }
                }
                files_to_parse.push((file.clone(), mtime));
            }
            for entry in &all_entries {
                if let (Some(mid), Some(rid)) = (&entry.message_id, &entry.request_id) {
                    seen.insert(DedupKey {
                        message_id: mid.clone(),
                        request_id: rid.clone(),
                    });
                }
            }
            for (file, mtime) in files_to_parse {
                let Ok(pf) = parse_jsonl_file(&file, &mut seen) else {
                    continue;
                };
                if let Some(mt) = mtime {
                    cache.entries.insert(file.path.clone(), (mt, pf.clone()));
                }
                all_entries.extend(pf.entries);
                all_limit_candidates.extend(pf.limit_candidates);
            }
        }
        None => {
            let parallel = parse_jsonl_files_parallel(&files, &mut seen);
            all_entries.extend(parallel.entries);
            all_limit_candidates.extend(parallel.limit_candidates);
        }
    }

    all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut blocks = transform_to_blocks(&all_entries, now);
    let limits = detect_limit_events_from_candidates(&all_limit_candidates, &mut blocks);
    let custom_limit = calculate_custom_limit(&blocks);
    let mut state = ReportState::from_blocks(now, blocks, limits);
    state.custom_limit = custom_limit;
    Ok(state)
}
