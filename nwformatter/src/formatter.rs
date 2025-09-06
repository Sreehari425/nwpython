//! Core NWPython formatter logic

pub fn format_nwpython(code: &str) -> String {
    let mut result = String::new();
    let mut indent = 0;
    let mut in_multiline_comment = false;
    let lines: Vec<&str> = code.lines().collect();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        // Handle multi-line comments
        if trimmed.starts_with("/*") {
            // Single-line block comment: "/* ... */"
            if trimmed.ends_with("*/") {
                result.push_str(&"    ".repeat(indent));
                result.push_str(trimmed);
                result.push('\n');
                continue;
            }
            // Print the opening line as-is and enter multi-line mode
            in_multiline_comment = true;
            result.push_str(&"    ".repeat(indent));
            result.push_str(trimmed);
            result.push('\n');
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
    // Handle inline or full-line single-line comments starting with // or #
    if let Some(pos) = trimmed.find("//").or_else(|| trimmed.find('#')) {
            let (left, right_with_slashes) = trimmed.split_at(pos);
            let code_part = left.trim_end();
            let comment_part = right_with_slashes.trim_start(); // keeps leading //
            result.push_str(&"    ".repeat(indent));
            if !code_part.is_empty() {
                result.push_str(code_part);
                if !code_part.ends_with(';') && !code_part.ends_with('{') && code_part != "}" {
                    result.push(';');
                }
                result.push(' ');
            }
            result.push_str(comment_part);
            result.push('\n');
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
