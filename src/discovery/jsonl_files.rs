use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JsonlFile {
    pub root: PathBuf,
    pub path: PathBuf,
}

/// Enumerates `.jsonl` files under `root` in deterministic path order.
/// When `since_threshold` is `Some`, files whose `mtime` is older than the
/// threshold are excluded without opening them. (ref: DL-003)
///
/// Returns an empty `Vec` when `root` is not a directory.
pub fn collect_jsonl_files(root: &Path, since_threshold: Option<SystemTime>) -> Vec<JsonlFile> {
    if !root.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect(root, root, since_threshold, &mut files);
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn collect(
    root: &Path,
    current: &Path,
    since_threshold: Option<SystemTime>,
    files: &mut Vec<JsonlFile>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, since_threshold, files);
            continue;
        }

        if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            if let Some(threshold) = since_threshold {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(mtime) = metadata.modified() {
                        if mtime < threshold {
                            continue;
                        }
                    }
                }
            }
            files.push(JsonlFile {
                root: root.to_path_buf(),
                path,
            });
        }
    }
}
