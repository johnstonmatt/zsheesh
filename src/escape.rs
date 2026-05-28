use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveAction {
    Skip,
    Off,
    On,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub line_number: usize,
    pub action: DirectiveAction,
}

pub fn extract_directives(input: &str) -> Vec<Directive> {
    let mut directives = Vec::new();
    for (i, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(comment) = trimmed.strip_prefix('#') {
            let body = comment.trim();
            if body == "fmt: skip" {
                directives.push(Directive {
                    line_number: i,
                    action: DirectiveAction::Skip,
                });
            } else if body == "fmt: off" {
                directives.push(Directive {
                    line_number: i,
                    action: DirectiveAction::Off,
                });
            } else if body == "fmt: on" {
                directives.push(Directive {
                    line_number: i,
                    action: DirectiveAction::On,
                });
            }
        }
    }
    directives
}

/// Replace protected regions with sentinel markers before formatting.
/// Returns the modified input and a map from sentinel line content to original lines.
pub fn protect_regions(input: &str) -> (String, BTreeMap<String, Vec<String>>) {
    let directives = extract_directives(input);
    if directives.is_empty() {
        return (input.to_owned(), BTreeMap::new());
    }

    let lines: Vec<&str> = input.lines().collect();
    let mut protected: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut region_id: usize = 0;

    let mut skip_set = std::collections::HashSet::new();
    let mut off_ranges: Vec<(usize, usize)> = Vec::new();

    for d in &directives {
        match d.action {
            DirectiveAction::Skip => {
                // The line immediately following `# fmt: skip` is protected
                let target = d.line_number + 1;
                if target < lines.len() {
                    skip_set.insert(target);
                }
            }
            DirectiveAction::Off => {
                // Find the matching `# fmt: on`
                let start = d.line_number;
                let end = directives
                    .iter()
                    .find(|other| other.line_number > start && other.action == DirectiveAction::On)
                    .map(|d| d.line_number)
                    .unwrap_or(lines.len());
                off_ranges.push((start, end));
            }
            DirectiveAction::On => {}
        }
    }

    let is_protected = |line_idx: usize| -> bool {
        if skip_set.contains(&line_idx) {
            return true;
        }
        for (start, end) in &off_ranges {
            if line_idx >= *start && line_idx <= *end {
                return true;
            }
        }
        false
    };

    let mut i = 0;
    while i < lines.len() {
        if is_protected(i) {
            let mut region_lines: Vec<String> = Vec::new();
            let region_start = i;
            while i < lines.len() && is_protected(i) {
                region_lines.push(lines[i].to_owned());
                i += 1;
            }
            let sentinel = format!("# __zsheesh_protected_region_{region_id}__ {region_start}");
            protected.insert(sentinel.clone(), region_lines);
            result_lines.push(sentinel);
            region_id += 1;
        } else {
            result_lines.push(lines[i].to_owned());
            i += 1;
        }
    }

    let mut output = result_lines.join("\n");
    // Preserve trailing newline
    if input.ends_with('\n') {
        output.push('\n');
    }
    (output, protected)
}

/// Protect specific line ranges (0-indexed, inclusive) by replacing them with sentinels.
/// Used for auto-protecting unparseable regions.
pub fn protect_line_ranges(
    input: &str,
    ranges: &[(usize, usize)],
) -> (String, BTreeMap<String, Vec<String>>) {
    if ranges.is_empty() {
        return (input.to_owned(), BTreeMap::new());
    }

    let lines: Vec<&str> = input.lines().collect();
    let mut protected: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut region_id: usize = 0;

    let is_in_range =
        |line_idx: usize| -> bool { ranges.iter().any(|(s, e)| line_idx >= *s && line_idx <= *e) };

    let mut i = 0;
    while i < lines.len() {
        if is_in_range(i) {
            let mut region_lines: Vec<String> = Vec::new();
            let region_start = i;
            while i < lines.len() && is_in_range(i) {
                region_lines.push(lines[i].to_owned());
                i += 1;
            }
            let sentinel = format!("# __zsheesh_protected_region_{region_id}__ {region_start}");
            protected.insert(sentinel.clone(), region_lines);
            result_lines.push(sentinel);
            region_id += 1;
        } else {
            result_lines.push(lines[i].to_owned());
            i += 1;
        }
    }

    let mut output = result_lines.join("\n");
    if input.ends_with('\n') {
        output.push('\n');
    }
    (output, protected)
}

/// Restore protected regions in the formatted output.
pub fn restore_regions(formatted: &str, protected: &BTreeMap<String, Vec<String>>) -> String {
    if protected.is_empty() {
        return formatted.to_owned();
    }

    let mut result = formatted.to_owned();
    for (sentinel, original_lines) in protected {
        let replacement = original_lines.join("\n");
        // The sentinel may have been reformatted — try to match flexibly
        if result.contains(sentinel) {
            result = result.replace(sentinel, &replacement);
        } else {
            // Try to find the sentinel by its unique ID portion
            let id_part = sentinel
                .split("__zsheesh_protected_region_")
                .nth(1)
                .and_then(|s| s.split("__").next());
            if let Some(id) = id_part {
                let marker = format!("__zsheesh_protected_region_{id}__");
                // Find line containing this marker and replace the whole line
                let mut new_lines: Vec<&str> = Vec::new();
                for line in result.lines() {
                    if line.contains(&marker) {
                        for orig in original_lines {
                            new_lines.push(orig);
                        }
                    } else {
                        new_lines.push(line);
                    }
                }
                let trailing = result.ends_with('\n');
                result = new_lines.join("\n");
                if trailing {
                    result.push('\n');
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_directives() {
        let input = "echo hello\necho world\n";
        let (output, protected) = protect_regions(input);
        assert_eq!(output, input);
        assert!(protected.is_empty());
    }

    #[test]
    fn fmt_skip_protects_next_line() {
        let input = "echo before\n# fmt: skip\necho   leave  me   alone\necho after\n";
        let directives = extract_directives(input);
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].action, DirectiveAction::Skip);
        assert_eq!(directives[0].line_number, 1);

        let (modified, protected) = protect_regions(input);
        assert_eq!(protected.len(), 1);
        // The protected region should contain the skip comment and the next line
        let region = protected.values().next().unwrap();
        assert!(region.iter().any(|l| l.contains("leave  me   alone")));
        // After restore, original content should be present
        let restored = restore_regions(&modified, &protected);
        assert!(restored.contains("leave  me   alone"));
    }

    #[test]
    fn fmt_off_on_region() {
        let input = "echo before\n# fmt: off\necho   a\necho   b\n# fmt: on\necho after\n";
        let (modified, protected) = protect_regions(input);
        assert_eq!(protected.len(), 1);
        let region = protected.values().next().unwrap();
        assert!(region.iter().any(|l| l.contains("echo   a")));
        assert!(region.iter().any(|l| l.contains("echo   b")));

        let restored = restore_regions(&modified, &protected);
        assert!(restored.contains("echo   a"));
        assert!(restored.contains("echo   b"));
    }

    #[test]
    fn fmt_off_without_on() {
        let input = "echo before\n# fmt: off\necho   a\necho   b\n";
        let (_, protected) = protect_regions(input);
        assert_eq!(protected.len(), 1);
        let region = protected.values().next().unwrap();
        assert!(region.iter().any(|l| l.contains("echo   a")));
        assert!(region.iter().any(|l| l.contains("echo   b")));
    }
}
