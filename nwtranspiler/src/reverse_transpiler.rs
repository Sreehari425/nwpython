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
        
        // Handle multi-line docstrings/comments
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let quote_type = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
            
            if trimmed.len() > 6 && trimmed.ends_with(quote_type) {
                // Single line docstring
                let content = &trimmed[3..trimmed.len()-3];
                result.push_str(&format!("/* {} */\n", content));
            } else {
                // Multi-line docstring
                result.push_str("/*\n");
                let content = &trimmed[3..];
                if !content.is_empty() {
                    result.push_str(&format!("{}\n", content));
                }
                
                i += 1;
                while i < lines.len() {
                    let next_line = lines[i];
                    let next_trimmed = next_line.trim();
                    if next_trimmed.ends_with(quote_type) {
                        let content = &next_trimmed[..next_trimmed.len()-3];
                        if !content.is_empty() {
                            result.push_str(&format!("{}\n", content));
                        }
                        result.push_str("*/\n");
                        break;
                    } else {
                        result.push_str(&format!("{}\n", next_trimmed));
                    }
                    i += 1;
                }
            }
            i += 1;
            continue;
        }
        
        // Handle single-line comments
        if trimmed.starts_with('#') {
            result.push_str(&format!("// {}\n", &trimmed[1..].trim()));
            i += 1;
            continue;
        }
        
        // Handle inline comments
        let (code_part, comment_part) = if let Some(hash_pos) = trimmed.find('#') {
            // Make sure # is not inside a string
            let before_hash = &trimmed[..hash_pos];
            if !is_inside_string(before_hash) {
                let code = before_hash.trim();
                let comment = &trimmed[hash_pos+1..].trim();
                (code, if comment.is_empty() { None } else { Some(*comment) })
            } else {
                (trimmed, None)
            }
        } else {
            (trimmed, None)
        };
        
        // Handle indentation changes (closing blocks)
        while current_indent < *indent_levels.last().unwrap() {
            indent_levels.pop();
            result.push_str("}\n");
        }
        
        if code_part.is_empty() {
            i += 1;
            continue;
        }
        
        let converted_code = convert_python_to_nw(code_part);
        
        // Check if this is a block header (ends with :)
        if code_part.ends_with(':') {
            result.push_str(&converted_code);
            if let Some(comment) = comment_part {
                result.push_str(&format!(" // {}", comment));
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
                result.push_str(&format!(" // {}", comment));
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
    
    // Convert def to function (but preserve the signature)
    let def_regex = Regex::new(r"^def\s+(\w+)\s*\((.*?)\)\s*(->\s*[^:]+)?:?$").unwrap();
    if let Some(caps) = def_regex.captures(&result) {
        let name = &caps[1];
        let params = &caps[2];
        let return_type = caps.get(3).map_or("", |m| m.as_str());
        result = format!("function {}({}){}", name, params, return_type);
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
    
    // Convert elif to else if
    if result.starts_with("elif ") {
        result = result.replace("elif ", "else if ");
    }
    
    // Convert Python operators to C-style where appropriate
    result = convert_operators(&result);
    
    result
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
    
    // Convert print() function calls to print statements  
    let print_regex = Regex::new(r"print\s*\((.*?)\)").unwrap();
    if let Some(caps) = print_regex.captures(&result) {
        let args = &caps[1];
        result = format!("print {}", args);
    }
    
    result
}
