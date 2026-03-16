use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JsonlFile {
    pub root: PathBuf,
    pub path: PathBuf,
}

pub fn collect_jsonl_files(root: &Path) -> Vec<JsonlFile> {
    if !root.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn collect(root: &Path, current: &Path, files: &mut Vec<JsonlFile>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
            continue;
        }

        if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            files.push(JsonlFile {
                root: root.to_path_buf(),
                path,
            });
        }
    }
}
