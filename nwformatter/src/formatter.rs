//! Core NWPython formatter logic

pub fn format_nwpython(code: &str) -> String {
    let mut result = String::new();
    let mut indent = 0;
    let mut in_multiline_comment = false;
    let lines: Vec<&str> = code.lines().collect();
    for line in lines {
        let trimmed = line.trim();
        // Handle multi-line comments
        if trimmed.starts_with("/*") {
            in_multiline_comment = true;
            result.push_str(&"    ".repeat(indent));
            result.push_str("/*\n");
            continue;
        }
        if in_multiline_comment {
            result.push_str(&"    ".repeat(indent));
            result.push_str(trimmed);
            result.push('\n');
            if trimmed.ends_with("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        // Handle block open
        if trimmed.ends_with("{") {
            result.push_str(&"    ".repeat(indent));
            result.push_str(trimmed);
            result.push('\n');
            indent += 1;
            continue;
        }
        // Handle block close
        if trimmed == "}" {
            indent = indent.saturating_sub(1);
            result.push_str(&"    ".repeat(indent));
            result.push_str("}\n");
            continue;
        }
        // Handle statements
        result.push_str(&"    ".repeat(indent));
        result.push_str(trimmed);
        if !trimmed.ends_with(';') && !trimmed.ends_with("{") && trimmed != "}" && !trimmed.is_empty() {
            result.push(';');
        }
        result.push('\n');
    }
    result
}
