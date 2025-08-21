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
                stmt_buf.push_str(s);
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
