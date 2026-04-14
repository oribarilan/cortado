//! Parser for Keep-a-Changelog formatted markdown.
//!
//! Extracts version entries between two semver versions so the update feed
//! can show users what changed since their current version.

use serde::Serialize;

/// A single version entry from the changelog.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangelogVersion {
    pub version: String,
    pub date: Option<String>,
    pub sections: Vec<ChangelogSection>,
}

/// A section within a version (Added, Changed, Fixed, etc.).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangelogSection {
    pub heading: String,
    pub entries: Vec<String>,
}

/// Parses a Keep-a-Changelog formatted string and returns version entries
/// where `from_version < version <= to_version`.
///
/// Both version strings are expected to be semver (with or without leading 'v').
/// Returns an empty vec on any parse error or if no versions match.
pub fn extract_range(
    changelog: &str,
    from_version: &str,
    to_version: &str,
) -> Vec<ChangelogVersion> {
    let from = match parse_semver(from_version) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let to = match parse_semver(to_version) {
        Some(v) => v,
        None => return Vec::new(),
    };

    if from >= to {
        return Vec::new();
    }

    let all_versions = parse_changelog(changelog);

    all_versions
        .into_iter()
        .filter(|entry| {
            if let Some(v) = parse_semver(&entry.version) {
                v > from && v <= to
            } else {
                false
            }
        })
        .collect()
}

/// Parses a full changelog string into a list of version entries.
fn parse_changelog(changelog: &str) -> Vec<ChangelogVersion> {
    let mut versions = Vec::new();
    let mut current_version: Option<ChangelogVersion> = None;
    let mut current_section: Option<ChangelogSection> = None;

    for line in changelog.lines() {
        let trimmed = line.trim();

        // Version header: ## [x.y.z] or ## [x.y.z] - date
        if let Some(version_header) = parse_version_header(trimmed) {
            // Flush current section into current version
            flush_section(&mut current_version, &mut current_section);
            // Flush current version
            if let Some(v) = current_version.take() {
                if !v.sections.is_empty() {
                    versions.push(v);
                }
            }
            current_version = Some(version_header);
            continue;
        }

        // Section header: ### Added, ### Changed, ### Fixed, etc.
        if let Some(heading) = parse_section_header(trimmed) {
            flush_section(&mut current_version, &mut current_section);
            current_section = Some(ChangelogSection {
                heading,
                entries: Vec::new(),
            });
            continue;
        }

        // Entry: - Some text
        if let Some(entry) = parse_entry(trimmed) {
            if let Some(section) = current_section.as_mut() {
                section.entries.push(entry);
            }
        }
    }

    // Flush remaining
    flush_section(&mut current_version, &mut current_section);
    if let Some(v) = current_version.take() {
        if !v.sections.is_empty() {
            versions.push(v);
        }
    }

    versions
}

fn flush_section(version: &mut Option<ChangelogVersion>, section: &mut Option<ChangelogSection>) {
    if let (Some(v), Some(s)) = (version.as_mut(), section.take()) {
        if !s.entries.is_empty() {
            v.sections.push(s);
        }
    }
}

/// Parses `## [x.y.z]` or `## [x.y.z] - 2026-04-14` into a ChangelogVersion.
/// Skips `## [Unreleased]`.
fn parse_version_header(line: &str) -> Option<ChangelogVersion> {
    if !line.starts_with("## ") {
        return None;
    }

    let rest = line[3..].trim();
    if !rest.starts_with('[') {
        return None;
    }

    let bracket_end = rest.find(']')?;
    let version_str = &rest[1..bracket_end];

    // Skip [Unreleased]
    if version_str.eq_ignore_ascii_case("unreleased") {
        return None;
    }

    let date = rest[bracket_end + 1..]
        .trim()
        .strip_prefix('-')
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());

    Some(ChangelogVersion {
        version: version_str.to_string(),
        date,
        sections: Vec::new(),
    })
}

/// Parses `### Added` into "Added".
fn parse_section_header(line: &str) -> Option<String> {
    if !line.starts_with("### ") {
        return None;
    }
    let heading = line[4..].trim().to_string();
    if heading.is_empty() {
        return None;
    }
    Some(heading)
}

/// Parses `- Some text` into "Some text".
fn parse_entry(line: &str) -> Option<String> {
    let stripped = line.strip_prefix("- ")?;
    let entry = stripped.trim().to_string();
    if entry.is_empty() {
        return None;
    }
    Some(entry)
}

fn parse_semver(version: &str) -> Option<semver::Version> {
    version
        .trim()
        .trim_start_matches('v')
        .parse::<semver::Version>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CHANGELOG: &str = r#"# Changelog

All notable changes to Cortado are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Fixed
- Something in progress

## [0.15.0] - 2026-04-15

### Added
- Terminal focus: cmux support
- Notification modes: "Worth Knowing" default

### Fixed
- Panel: empty state hotkey hint shows actual shortcut

## [0.14.0] - 2026-04-14

### Changed
- Settings: feed name placeholder matches the feed type

### Fixed
- GitHub Actions: cancelled runs are now passive

## [0.13.0] - 2026-04-13

### Fixed
- Panel: install-update buttons work in the panel
- Settings: tools installed via ~/.zshrc detected correctly
- Notifications: agent feeds fire notifications reliably

## [0.12.0] - 2026-04-12

### Added
- Config change detection with one-click restart

### Fixed
- Focus session works with tmux inside Ghostty
"#;

    #[test]
    fn extract_single_version() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.14.0", "0.15.0");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "0.15.0");
        assert_eq!(result[0].date.as_deref(), Some("2026-04-15"));
        assert_eq!(result[0].sections.len(), 2);
        assert_eq!(result[0].sections[0].heading, "Added");
        assert_eq!(result[0].sections[0].entries.len(), 2);
        assert_eq!(result[0].sections[1].heading, "Fixed");
        assert_eq!(result[0].sections[1].entries.len(), 1);
    }

    #[test]
    fn extract_multiple_versions() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.12.0", "0.15.0");
        assert_eq!(result.len(), 3);
        // Newest first (order preserved from changelog)
        assert_eq!(result[0].version, "0.15.0");
        assert_eq!(result[1].version, "0.14.0");
        assert_eq!(result[2].version, "0.13.0");
    }

    #[test]
    fn extract_excludes_from_version() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.13.0", "0.15.0");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].version, "0.15.0");
        assert_eq!(result[1].version, "0.14.0");
        // 0.13.0 should NOT be included (exclusive lower bound)
    }

    #[test]
    fn extract_includes_to_version() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.13.0", "0.14.0");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "0.14.0");
    }

    #[test]
    fn extract_same_version_returns_empty() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.14.0", "0.14.0");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_from_greater_than_to_returns_empty() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.15.0", "0.14.0");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_no_matching_versions() {
        let result = extract_range(SAMPLE_CHANGELOG, "1.0.0", "2.0.0");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_skips_unreleased() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.14.0", "999.0.0");
        // Should include 0.15.0 but NOT Unreleased
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "0.15.0");
    }

    #[test]
    fn extract_with_v_prefix() {
        let result = extract_range(SAMPLE_CHANGELOG, "v0.14.0", "v0.15.0");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "0.15.0");
    }

    #[test]
    fn extract_invalid_from_version_returns_empty() {
        let result = extract_range(SAMPLE_CHANGELOG, "not-semver", "0.15.0");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_invalid_to_version_returns_empty() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.14.0", "not-semver");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_empty_changelog() {
        let result = extract_range("", "0.14.0", "0.15.0");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_malformed_changelog() {
        let input = "This is not a changelog\nJust random text\n## No brackets\n### ??? ";
        let result = extract_range(input, "0.0.0", "99.0.0");
        assert!(result.is_empty());
    }

    #[test]
    fn section_entries_are_correct() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.12.0", "0.13.0");
        assert_eq!(result.len(), 1);
        let v = &result[0];
        assert_eq!(v.version, "0.13.0");
        assert_eq!(v.sections.len(), 1);
        assert_eq!(v.sections[0].heading, "Fixed");
        assert_eq!(v.sections[0].entries.len(), 3);
        assert_eq!(
            v.sections[0].entries[0],
            "Panel: install-update buttons work in the panel"
        );
    }

    #[test]
    fn version_with_multiple_sections() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.13.0", "0.14.0");
        let v = &result[0];
        assert_eq!(v.sections.len(), 2);
        assert_eq!(v.sections[0].heading, "Changed");
        assert_eq!(v.sections[1].heading, "Fixed");
    }

    #[test]
    fn parse_version_header_basic() {
        let h = parse_version_header("## [1.2.3]").unwrap();
        assert_eq!(h.version, "1.2.3");
        assert!(h.date.is_none());
    }

    #[test]
    fn parse_version_header_with_date() {
        let h = parse_version_header("## [1.2.3] - 2026-04-14").unwrap();
        assert_eq!(h.version, "1.2.3");
        assert_eq!(h.date.as_deref(), Some("2026-04-14"));
    }

    #[test]
    fn parse_version_header_unreleased_skipped() {
        assert!(parse_version_header("## [Unreleased]").is_none());
        assert!(parse_version_header("## [UNRELEASED]").is_none());
    }

    #[test]
    fn parse_version_header_not_a_header() {
        assert!(parse_version_header("# Not a version").is_none());
        assert!(parse_version_header("### Not a version").is_none());
        assert!(parse_version_header("## No brackets").is_none());
    }

    #[test]
    fn parse_section_header_valid() {
        assert_eq!(parse_section_header("### Added"), Some("Added".to_string()));
        assert_eq!(parse_section_header("### Fixed"), Some("Fixed".to_string()));
    }

    #[test]
    fn parse_section_header_invalid() {
        assert!(parse_section_header("## Not h3").is_none());
        assert!(parse_section_header("### ").is_none());
    }

    #[test]
    fn parse_entry_valid() {
        assert_eq!(
            parse_entry("- Some feature"),
            Some("Some feature".to_string())
        );
    }

    #[test]
    fn parse_entry_empty_after_dash() {
        assert!(parse_entry("- ").is_none());
        assert!(parse_entry("-").is_none());
    }

    #[test]
    fn parse_entry_not_an_entry() {
        assert!(parse_entry("Not a list item").is_none());
        assert!(parse_entry("* Star bullet").is_none());
    }

    #[test]
    fn empty_sections_are_excluded() {
        let input = "## [1.0.0]\n\n### Added\n\n### Fixed\n- A fix\n";
        let result = extract_range(input, "0.0.0", "1.0.0");
        assert_eq!(result.len(), 1);
        // Added section has no entries, should be excluded
        assert_eq!(result[0].sections.len(), 1);
        assert_eq!(result[0].sections[0].heading, "Fixed");
    }

    #[test]
    fn versions_with_no_sections_are_excluded() {
        let input =
            "## [1.0.0]\n\nJust some text, no sections\n\n## [0.9.0]\n### Added\n- A thing\n";
        let result = extract_range(input, "0.0.0", "1.0.0");
        // 1.0.0 has no sections with entries, so excluded
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "0.9.0");
    }

    #[test]
    fn serializes_to_json() {
        let result = extract_range(SAMPLE_CHANGELOG, "0.14.0", "0.15.0");
        let json = serde_json::to_string(&result).expect("serializes");
        assert!(json.contains("0.15.0"));
        assert!(json.contains("cmux"));
    }
}
