use std::collections::HashMap;

/// Extract raw markdown sections from a maskfile, keyed by command name.
/// Handles nested headings for subcommands (e.g. "parent subcommand").
pub fn extract_sections(content: &str) -> HashMap<String, String> {
    let mut sections: HashMap<String, String> = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();
    let mut breadcrumb: Vec<(u8, String)> = vec![];

    for line in content.lines() {
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
}
