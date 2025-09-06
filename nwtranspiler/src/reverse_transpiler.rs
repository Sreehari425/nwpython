//! Reverse transpiler: Python → NWPython
//! Converts standard Python code to NWPython curly-brace/semicolon syntax

use regex::Regex;

pub fn reverse_transpile(py_code: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = py_code.lines().collect();
    let mut indent_levels: Vec<usize> = vec![0];
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let current_indent = line.len() - line.trim_start().len();
        
        // Skip empty lines
        if trimmed.is_empty() {
            result.push('\n');
            i += 1;
            continue;
        }

        // Close any blocks before handling comments/docstrings at a lower indent
        while current_indent < *indent_levels.last().unwrap() {
            indent_levels.pop();
            result.push_str("}\n");
        }
        
        // Handle multi-line docstrings/comments
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let quote_type = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
            // Single-line docstring
            if trimmed.ends_with(quote_type) && trimmed.len() >= 6 {
                let content = &trimmed[3..trimmed.len()-3];
                result.push_str(&format!("/* {} */\n", content));
                i += 1;
                continue;
            }
            // Multi-line docstring: consume until the closing quotes
            result.push_str("/*\n");
            let first_content = &trimmed[3..];
            if !first_content.is_empty() {
                result.push_str(first_content);
                result.push('\n');
            }
            let mut j = i + 1;
            let mut closed = false;
            while j < lines.len() {
                let next_line = lines[j];
                let next_trimmed = next_line.trim();
                if next_trimmed.ends_with(quote_type) {
                    let body = &next_trimmed[..next_trimmed.len()-3];
                    if !body.is_empty() {
                        result.push_str(body);
                        result.push('\n');
                    }
                    result.push_str("*/\n");
                    closed = true;
                    j += 1; // move past the closing line
                    break;
                } else {
                    result.push_str(next_trimmed);
                    result.push('\n');
                }
                j += 1;
            }
            if !closed {
                // No closing quotes found; close the comment to avoid leaking
                result.push_str("*/\n");
            }
            i = j;
            continue;
        }
        
        // Handle single-line comments (keep '#')
        if trimmed.starts_with('#') {
            result.push_str(&format!("# {}\n", &trimmed[1..].trim()));
            i += 1;
            continue;
        }
        
        // Handle inline comments
    let (code_part, comment_part) = if let Some(hash_pos) = trimmed.find('#') {
            // Make sure # is not inside a string
            let before_hash = &trimmed[..hash_pos];
            if !is_inside_string(before_hash) {
                let code = before_hash.trim();
        let comment = trimmed[hash_pos+1..].trim();
        (code, if comment.is_empty() { None } else { Some(comment) })
            } else {
                (trimmed, None)
            }
        } else {
            (trimmed, None)
        };
        
        if code_part.is_empty() {
            i += 1;
            continue;
        }
        
    let converted_code = convert_python_to_nw(code_part);
        
        // Check if this is a block header (ends with :)
    if code_part.ends_with(':') {
            result.push_str(&converted_code);
            if let Some(comment) = comment_part {
                result.push_str(&format!(" # {}", comment));
            }
            result.push_str(" {\n");
            // Calculate next expected indent
            if i + 1 < lines.len() {
                let next_line = lines[i + 1];
                let next_indent = next_line.len() - next_line.trim_start().len();
                if next_indent > current_indent {
                    indent_levels.push(next_indent);
                } else {
                    indent_levels.push(current_indent + 4);
                }
            } else {
                indent_levels.push(current_indent + 4);
            }
        } else {
            result.push_str(&converted_code);
            result.push(';');
            if let Some(comment) = comment_part {
                result.push_str(&format!(" # {}", comment));
            }
            result.push('\n');
        }
        
        i += 1;
    }
    
    // Close remaining blocks
    while indent_levels.len() > 1 {
        indent_levels.pop();
        result.push_str("}\n");
    }
    
    result
}

fn is_inside_string(code: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    
    for ch in code.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        
        match ch {
            '\\' => escaped = true,
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            _ => {}
        }
    }
    
    in_single || in_double
}

fn convert_python_to_nw(code: &str) -> String {
    let mut result = code.to_string();
    
    // Normalize Python def: keep 'def', strip param type annotations but keep return type if present
    let def_regex = Regex::new(r"^def\s+(\w+)\s*\((.*?)\)\s*(->\s*[^:]+)?:?$").unwrap();
    if let Some(caps) = def_regex.captures(&result) {
        let name = &caps[1];
        let params = &caps[2];
        let cleaned_params = strip_param_types(params);
        let ret = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");
        if ret.is_empty() {
            result = format!("def {}({})", name, cleaned_params);
        } else {
            result = format!("def {}({}) {}", name, cleaned_params, ret);
        }
        return result;
    }
    
    // Convert class
    let class_regex = Regex::new(r"^class\s+(\w+).*:?$").unwrap();
    if let Some(caps) = class_regex.captures(&result) {
        let name = &caps[1];
        result = format!("class {}", name);
        return result;
    }
    
    // Convert control structures (if, elif, else, while, for, etc.)
    if result.ends_with(':') {
        result.pop(); // Remove the colon
    }
    
    // Keep 'elif' as-is (closer to NWPython example)
    
    // Wrap conditions in parentheses for if/elif/while/for
    for kw in ["if", "elif", "while", "for"].iter() {
        let prefix = format!("{} ", kw);
        if result.starts_with(&prefix) {
            let cond = result[prefix.len()..].trim();
            result = format!("{} ({})", kw, cond);
            break;
        }
    }
    
    // Convert Python operators to C-style where appropriate
    result = convert_operators(&result);
    
    result
}

fn strip_param_types(params: &str) -> String {
    // Split by commas at top-level (no deep parsing); strip annotations like `name: Type` but keep defaults `= value`.
    params
        .split(',')
        .map(|p| {
            let p = p.trim();
            if p.is_empty() { return String::new(); }
            let (before_eq, after_eq_opt) = if let Some(eq_pos) = p.find('=') {
                (p[..eq_pos].trim(), Some(p[eq_pos+1..].trim()))
            } else { (p, None) };
            // Remove annotation after ':' in before_eq
            let name_part = if let Some(colon_pos) = before_eq.find(':') {
                before_eq[..colon_pos].trim()
            } else { before_eq };
            match after_eq_opt {
                Some(def_val) if !def_val.is_empty() => format!("{} = {}", name_part, def_val),
                _ => name_part.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn convert_operators(code: &str) -> String {
    let mut result = code.to_string();
    
    // Convert += 1 to ++
    let inc_regex = Regex::new(r"(\w+)\s*\+=\s*1\b").unwrap();
    result = inc_regex.replace_all(&result, "$1++").to_string();
    
    // Convert -= 1 to --
    let dec_regex = Regex::new(r"(\w+)\s*-=\s*1\b").unwrap();
    result = dec_regex.replace_all(&result, "$1--").to_string();
    
    // Handle special case: n += 1 in return statement
    let return_inc_regex = Regex::new(r"return\s+(\w+)\s*\+=\s*1").unwrap();
    result = return_inc_regex.replace_all(&result, "return ++$1").to_string();
    
    // Keep print() as-is for NWPython
    
    result
}
