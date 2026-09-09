//! Bundled path-to-fileDataID lookup for fresh CASC cache population.
//!
//! This is intentionally a small subset of the community listfile: paths the
//! simulator is expected to request during normal UI rendering.

use std::collections::HashMap;
use std::sync::LazyLock;

const BUNDLED_LISTFILE: &str = include_str!("../data/wow-ui-sim-listfile.csv");

static BY_PATH: LazyLock<HashMap<String, ListfileEntry>> = LazyLock::new(parse_bundled_listfile);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListfileEntry {
    pub fdid: u32,
    pub path: &'static str,
}

pub fn lookup_path(path: &str) -> Option<u32> {
    lookup_entry(path).map(|entry| entry.fdid)
}

pub fn lookup_entry(path: &str) -> Option<ListfileEntry> {
    let normalized = normalize_path(path);
    BY_PATH.get(normalized.as_str()).copied()
}

pub fn lookup_texture_path(path: &str) -> Option<u32> {
    let normalized = normalize_path(path);
    let without_extension = normalized
        .rsplit_once('.')
        .map(|(path, _)| path)
        .unwrap_or(&normalized);
    let prefixed = without_extension.strip_prefix("interface/").map_or_else(
        || format!("interface/{without_extension}"),
        ToString::to_string,
    );
    [
        normalized.clone(),
        format!("{without_extension}.blp"),
        format!("{prefixed}.blp"),
    ]
    .into_iter()
    .find_map(|candidate| BY_PATH.get(&candidate).map(|entry| entry.fdid))
}

pub fn entries() -> impl Iterator<Item = ListfileEntry> {
    BY_PATH.values().copied()
}

pub fn blizzard_ui_entries() -> impl Iterator<Item = ListfileEntry> {
    entries().filter(|entry| entry.path.starts_with("interface/addons/blizzard_"))
}

fn parse_bundled_listfile() -> HashMap<String, ListfileEntry> {
    BUNDLED_LISTFILE
        .lines()
        .filter_map(parse_row)
        .map(|entry| (normalize_path(entry.path), entry))
        .collect()
}

fn parse_row(row: &'static str) -> Option<ListfileEntry> {
    let (fdid, path) = row.trim_end_matches('\r').split_once(';')?;
    let fdid = fdid.parse().ok()?;
    Some(ListfileEntry { fdid, path })
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{blizzard_ui_entries, lookup_path};

    #[test]
    fn lookup_path_normalizes_slashes_and_case() {
        assert_eq!(
            lookup_path("Interface\\Icons\\Trade_Engineering.blp"),
            Some(136243)
        );
    }

    #[test]
    fn blizzard_ui_entries_include_only_addon_tree_rows() {
        let entries: Vec<_> = blizzard_ui_entries().take(3).collect();

        assert!(!entries.is_empty());
        assert!(
            entries
                .iter()
                .all(|entry| entry.path.starts_with("interface/addons/blizzard_"))
        );
    }
}
