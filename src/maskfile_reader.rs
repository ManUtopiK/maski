use std::collections::HashMap;

/// Extract raw markdown sections from a maskfile, keyed by command name.
/// Handles nested headings for subcommands (e.g. "parent subcommand").
pub fn extract_sections(content: &str) -> HashMap<String, String> {
    let mut sections: HashMap<String, String> = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();
    let mut breadcrumb: Vec<(u8, String)> = vec![];
    // Track fenced code blocks: lines inside them (e.g. bash `# comment`) must
    // never be parsed as headings, or they corrupt the breadcrumb.
    let mut fence: Option<char> = None;

    for line in content.lines() {
        if let Some(marker) = fence_marker(line) {
            match fence {
                Some(open) if open == marker => fence = None, // closing fence
                Some(_) => {}                                 // different marker inside fence
                None => fence = Some(marker),                 // opening fence
            }
            current_body.push_str(line);
            current_body.push('\n');
            continue;
        }

        // Inside a code block: accumulate verbatim, never parse as a heading.
        if fence.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
            continue;
        }

        if let Some((level, raw_name)) = parse_heading(line) {
            // Save previous section
            if let Some(name) = current_name.take() {
                sections.insert(name, current_body.trim().to_string());
            }

            // Strip args from heading: "start [name]" -> "start"
            let cmd_name = raw_name
                .split(|c: char| c == '(' || c == '[')
                .next()
                .unwrap_or(&raw_name)
                .trim()
                .to_string();

            // Level 1 is the title, skip
            if level == 1 {
                current_name = None;
                current_body = String::new();
                breadcrumb.clear();
                continue;
            }

            // Build full path for subcommands
            // Remove entries at same or deeper level
            while breadcrumb.last().is_some_and(|(l, _)| *l >= level) {
                breadcrumb.pop();
            }

            // mask strips parent name prefix from subcommands
            // e.g. "### parent subcommand" under "## parent" becomes just "subcommand"
            let stripped_name = if let Some((_, parent_name)) = breadcrumb.last() {
                cmd_name
                    .strip_prefix(parent_name.as_str())
                    .map(|s| s.trim().to_string())
                    .unwrap_or(cmd_name.clone())
            } else {
                cmd_name.clone()
            };

            breadcrumb.push((level, stripped_name));

            // The key is the full path: "parent > subcommand"
            let full_name = breadcrumb
                .iter()
                .map(|(_, n)| n.as_str())
                .collect::<Vec<_>>()
                .join(" > ");

            current_name = Some(full_name);
            current_body = String::new();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    // Save last section
    if let Some(name) = current_name {
        sections.insert(name, current_body.trim().to_string());
    }

    sections
}

/// Detect a fenced code block delimiter (``` or ~~~), returning its marker char.
/// Markdown allows leading whitespace before the fence.
fn fence_marker(line: &str) -> Option<char> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some('`')
    } else if trimmed.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = trimmed[level..].trim();
    if rest.is_empty() {
        return None;
    }
    Some((level as u8, rest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sections() {
        let content = r#"# Title

Some intro

## start [name]

> Start a container

Some extra text.

```bash
echo hello
```

## stop [name]

> Stop it

```bash
echo bye
```
"#;
        let sections = extract_sections(content);
        assert!(sections.contains_key("start"));
        assert!(sections.contains_key("stop"));
        assert!(sections["start"].contains("Some extra text."));
        assert!(sections["start"].contains("```bash"));
    }

    #[test]
    fn test_subcommands() {
        let content = r#"# Title

## parent

### parent subcommand

> A subcommand

```bash
echo hey
```
"#;
        let sections = extract_sections(content);
        assert!(sections.contains_key("parent > subcommand"));
    }

    // Regression: bash `# comment` lines inside a subcommand's code block were
    // parsed as level-1 headings, clearing the breadcrumb. Subcommands that
    // followed lost their parent prefix and overwrote same-named top-level
    // commands (e.g. `vm > rebuild` collided with top-level `rebuild`).
    #[test]
    fn test_comments_in_code_block_dont_corrupt_breadcrumb() {
        let content = r#"# Title

## rebuild (client)

> Top-level rebuild

```bash
echo "build and push"
```

## vm

### browse (app)

> Browse, with hash comments in the script

```bash
# 1. start the proxy
# 2. resolve the domain
# 3. open the browser
echo browse
```

### rebuild (name)

> Stop + clean + build + run

```bash
echo "vm rebuild"
```
"#;
        let sections = extract_sections(content);

        // Both rebuilds must coexist under distinct keys.
        assert!(sections.contains_key("rebuild"), "top-level rebuild missing");
        assert!(sections.contains_key("vm > rebuild"), "vm > rebuild missing");
        assert!(sections.contains_key("vm > browse"), "vm > browse missing");

        // The top-level rebuild must NOT be overwritten by the vm subcommand.
        assert!(
            sections["rebuild"].contains("Top-level rebuild"),
            "top-level rebuild was clobbered: {:?}",
            sections["rebuild"]
        );
        assert!(sections["vm > rebuild"].contains("Stop + clean + build + run"));

        // The hash comments must stay in the browse body, not become headings.
        assert!(sections["vm > browse"].contains("# 1. start the proxy"));
        assert!(!sections.contains_key("1. start the proxy"));
    }
}
