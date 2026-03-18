//! Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub enum RootSource {
    Standard,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub struct DiscoveredRoot {
    pub path: PathBuf,
    pub source: RootSource,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub struct RootDiscovery {
    pub roots: Vec<DiscoveredRoot>,
    pub selected: Option<PathBuf>,
}

/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub fn standard_roots() -> Vec<DiscoveredRoot> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    vec![
        DiscoveredRoot {
            path: home.join(".claude/projects"),
            source: RootSource::Standard,
        },
        DiscoveredRoot {
            path: home.join(".config/claude/projects"),
            source: RootSource::Standard,
        },
    ]
}

/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub fn discover_roots() -> RootDiscovery {
    discover_roots_with(&[])
}

/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub fn discover_roots_with(custom_roots: &[PathBuf]) -> RootDiscovery {
    let candidates = if custom_roots.is_empty() {
        standard_roots()
    } else {
        custom_roots
            .iter()
            .cloned()
            .map(|path| DiscoveredRoot {
                path,
                source: RootSource::Custom,
            })
            .collect()
    };

    let mut seen = BTreeSet::new();
    let mut roots = Vec::new();
    for candidate in candidates {
        if candidate.path.is_dir() && seen.insert(candidate.path.clone()) {
            roots.push(candidate);
        }
    }

    roots.sort_by(|left, right| left.path.cmp(&right.path));
    let selected = roots.first().map(|root| root.path.clone());

    RootDiscovery { roots, selected }
}

/// Root discovery preserves upstream selection rules while fixture coverage audits edge cases. (ref: DL-001) (ref: DL-005)
pub fn select_primary_root(discovery: &RootDiscovery) -> Option<&Path> {
    // Preserve the upstream first-root selection until fixture evidence
    // proves a safe multi-root behavior change.
    discovery.selected.as_deref()
}

pub fn select_roots(discovery: &RootDiscovery) -> Vec<&Path> {
    discovery
        .roots
        .iter()
        .map(|root| root.path.as_path())
        .collect()
}
