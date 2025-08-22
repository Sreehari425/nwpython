//! Transpiler: convert tokens to Python code
use nwparser::tokenizer::Token;

pub fn transpile(tokens: &[Token]) -> String {
    let mut out_lines = Vec::new();
    let mut indent = 0;
    let mut stmt_buf = String::new();
    let block_headers = ["if ", "elif ", "else", "def ", "while ", "for "];

    fn header_needs_colon(s: &str, block_headers: &[&str]) -> bool {
        let s = s.trim();
        block_headers.iter().any(|h| s.starts_with(h))
    }

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment(c) => {
                let c = c.trim_start_matches(['/', '#', ' ']);
                if c.starts_with("*") {
                    // Multi-line comment: /* ... */
                    let content = c.trim_start_matches('*').trim_start_matches('/').trim_end_matches("*/").trim();
                    out_lines.push(format!("{}\"\"\"{}\"\"\"", "    ".repeat(indent), content));
                } else {
                    // Single-line comment
                    out_lines.push(format!("{}# {}", "    ".repeat(indent), c));
                }
            }
            Token::Text(s) => {
                let mut line = s.to_string();
                let re_print_post_inc = regex::Regex::new(r"print\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\+\+\s*\)").unwrap();
                let re_print_post_dec = regex::Regex::new(r"print\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*--\s*\)").unwrap();
                let re_print_pre_inc = regex::Regex::new(r"print\s*\(\s*\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)").unwrap();
                let re_print_pre_dec = regex::Regex::new(r"print\s*\(\s*--\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)").unwrap();
                let mut handled = false;
                if re_print_post_inc.is_match(&line) {
                    let var = re_print_post_inc.captures(&line).unwrap().get(1).unwrap().as_str();
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_post_dec.is_match(&line) {
                    let var = re_print_post_dec.captures(&line).unwrap().get(1).unwrap().as_str();
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_pre_inc.is_match(&line) {
                    let var = re_print_pre_inc.captures(&line).unwrap().get(1).unwrap().as_str();
                    out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_pre_dec.is_match(&line) {
                    let var = re_print_pre_dec.captures(&line).unwrap().get(1).unwrap().as_str();
                    out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    handled = true;
                }
                if handled {
                    // Don't process further
                } else {
                    // Handle return x++ and return x-- (post-increment/decrement)
                    let re_ret_pre_inc = regex::Regex::new(r"return\s*\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                    let re_ret_pre_dec = regex::Regex::new(r"return\s*--\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                    let mut handled_ret = false;
                    if re_ret_pre_inc.is_match(&line) {
                        let var = re_ret_pre_inc.captures(&line).unwrap().get(1).unwrap().as_str();
                        stmt_buf.push_str("");
                        out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                        out_lines.push(format!("{}return {}", "    ".repeat(indent), var));
                        handled_ret = true;
                    }
                    if re_ret_pre_dec.is_match(&line) {
                        let var = re_ret_pre_dec.captures(&line).unwrap().get(1).unwrap().as_str();
                        stmt_buf.push_str("");
                        out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                        out_lines.push(format!("{}return {}", "    ".repeat(indent), var));
                        handled_ret = true;
                    }
                    if handled_ret {
                        // Don't process further
                    } else {
                        // Handle ++x and --x (standalone)
                        let re_pre_inc = regex::Regex::new(r"\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                        let re_pre_dec = regex::Regex::new(r"--\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                        line = re_pre_inc.replace_all(&line, "$1 += 1").to_string();
                        line = re_pre_dec.replace_all(&line, "$1 -= 1").to_string();
                        // Handle x++ and x-- (standalone)
                        let re_inc = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\+\+").unwrap();
                        let re_dec = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*--").unwrap();
                        line = re_inc.replace_all(&line, "$1 += 1").to_string();
                        line = re_dec.replace_all(&line, "$1 -= 1").to_string();
                        stmt_buf.push_str(&line);
                    }
                }
            }
            Token::LBrace => {
                let header = stmt_buf.trim().to_string();
                stmt_buf.clear();
                if header.is_empty() {
                    out_lines.push("# ERROR: Found '{' without header".to_string());
                } else {
                    let mut line = header;
                    if !line.ends_with(':') {
                        line.push(':');
                    }
                    out_lines.push(format!("{}{}", "    ".repeat(indent), line));
                    indent += 1;
                }
            }
            Token::RBrace => {
                let simple = stmt_buf.trim().to_string();
                if !simple.is_empty() {
                    for stmt in simple.split(';') {
                        let s = stmt.trim();
                        if !s.is_empty() {
                            out_lines.push(format!("{}{}", "    ".repeat(indent), s));
                        }
                    }
                    stmt_buf.clear();
                }
                if indent > 0 {
                    indent -= 1;
                } else {
                    out_lines.push("# ERROR: Too many '}'".to_string());
                }
            }
            Token::Semicolon => {
                let stmt = stmt_buf.trim().to_string();
                stmt_buf.clear();
                for s in stmt.split(';') {
                    let s = s.trim();
                    if !s.is_empty() {
                        if header_needs_colon(s, &block_headers) {
                            let mut line = s.to_string();
                            if !line.ends_with(':') {
                                line.push(':');
                            }
                            out_lines.push(format!("{}{}", "    ".repeat(indent), line));
                            indent += 1;
                        } else {
                            out_lines.push(format!("{}{}", "    ".repeat(indent), s));
                        }
                    }
                }
            }
        }
        i += 1;
    }
    let tail = stmt_buf.trim();
    if !tail.is_empty() {
        for s in tail.split(';') {
            let s = s.trim();
            if !s.is_empty() {
                if header_needs_colon(s, &block_headers) {
                    let mut line = s.to_string();
                    if !line.ends_with(':') {
                        line.push(':');
                    }
                    out_lines.push(format!("{}{}", "    ".repeat(indent), line));
                } else {
                    out_lines.push(format!("{}{}", "    ".repeat(indent), s));
                }
            }
        }
    }
    out_lines.join("\n") + "\n"
}
