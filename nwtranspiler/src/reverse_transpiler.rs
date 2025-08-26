//! Reverse transpiler: Python → NWPython
//! Converts standard Python code to NWPython curly-brace/semicolon syntax

use regex::Regex;

pub fn reverse_transpile(py_code: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = py_code.lines().collect();
    let mut indent_stack: Vec<usize> = vec![0];

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let current_indent = line.len() - line.trim_start().len();

        // Skip empty lines
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }

        // Handle comments
        if trimmed.starts_with('#') {
            result.push_str(&format!("// {}\n", &trimmed[1..].trim()));
            continue;
        }

        // Handle multi-line docstrings
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let quote_type = if trimmed.starts_with("\"\"\"") {
                "\"\"\""
            } else {
                "'''"
            };

            if trimmed.len() > 6 && trimmed.ends_with(quote_type) {
                // Single line docstring
                let content = &trimmed[3..trimmed.len() - 3];
                result.push_str(&format!("/* {} */\n", content));
            } else {
                // Multi-line docstring - convert to multi-line comment
                result.push_str("/*\n");
                let first_content = &trimmed[3..];
                if !first_content.is_empty() {
                    result.push_str(&format!("{}\n", first_content));
                }

                // Find closing quotes in subsequent lines (this is incomplete but works for simple cases)
                result.push_str("*/\n");
            }
            continue;
        }

        // Handle indentation changes (close blocks)
        while current_indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            result.push_str("}\n");
        }

        // Convert the line
        let converted = convert_python_line(trimmed);

        // Check if this starts a new block (ends with :)
        if trimmed.ends_with(':') {
            result.push_str(&converted);
            result.push_str(" {\n");

            // Predict next indent level
            if line_idx + 1 < lines.len() {
                let next_line = lines[line_idx + 1];
                let next_indent = next_line.len() - next_line.trim_start().len();
                if next_indent > current_indent {
                    indent_stack.push(next_indent);
                } else {
                    indent_stack.push(current_indent + 4);
                }
            }
        } else {
            result.push_str(&converted);
            result.push_str(";\n");
        }
    }

    // Close any remaining blocks
    while indent_stack.len() > 1 {
        indent_stack.pop();
        result.push_str("}\n");
    }

    result
}

fn convert_python_line(line: &str) -> String {
    let mut result = line.to_string();

    // Convert def to function
    if let Some(caps) = Regex::new(r"^def\s+(\w+)\s*\((.*?)\)\s*(->\s*[^:]+)?:?$")
        .unwrap()
        .captures(&result)
    {
        let name = &caps[1];
        let params = &caps[2];
        let return_type = caps.get(3).map_or("", |m| m.as_str());
        return format!("def {}({}){}", name, params, return_type);
    }

    // Convert class
    if let Some(caps) = Regex::new(r"^class\s+(\w+).*:?$")
        .unwrap()
        .captures(&result)
    {
        let name = &caps[1];
        return format!("class {}", name);
    }

    // Convert elif to else if
    if result.starts_with("elif ") {
        result = result.replace("elif ", "else if ");
    }

    // Remove trailing colon from control structures
    if result.ends_with(':') {
        result.pop();
    }

    // Convert operators
    result = convert_operators(&result);

    result
}

fn convert_operators(code: &str) -> String {
    let mut result = code.to_string();

    // Convert += 1 to ++
    result = Regex::new(r"(\w+)\s*\+=\s*1\b")
        .unwrap()
        .replace_all(&result, "$1++")
        .to_string();

    // Convert -= 1 to --
    result = Regex::new(r"(\w+)\s*-=\s*1\b")
        .unwrap()
        .replace_all(&result, "$1--")
        .to_string();

    // Convert print() calls to print statements
    result = Regex::new(r"print\s*\((.*?)\)")
        .unwrap()
        .replace_all(&result, "print $1")
        .to_string();

    result
}
