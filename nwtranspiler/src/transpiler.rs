//! Transpiler: convert tokens to Python code
use nwparser::tokenizer::Token;

pub fn transpile(tokens: &[Token]) -> String {
    let mut out_lines = Vec::new();
    let mut indent = 0;
    let mut stmt_buf = String::new();

    fn emit_stmt(s: &str, indent: usize, out: &mut Vec<String>) {
        let line = format!("{}{}", "    ".repeat(indent), s.trim());
        out.push(line);
    }

    fn emit_header(s: &str, indent: usize, out: &mut Vec<String>) {
        let mut line = s.trim().to_string();
        if !line.ends_with(':') {
            line.push(':');
        }
        out.push(format!("{}{}", "    ".repeat(indent), line));
    }

    let block_headers = ["if ", "elif ", "else", "def ", "while ", "for "];

    fn header_needs_colon(s: &str, block_headers: &[&str]) -> bool {
        let s = s.trim();
        block_headers.iter().any(|h| s.starts_with(h))
    }

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::LBrace => {
                let header = stmt_buf.trim().to_string();
                stmt_buf.clear();
                if header.is_empty() {
                    out_lines.push("# ERROR: Found '{' without header".to_string());
                } else {
                    emit_header(&header, indent, &mut out_lines);
                    indent += 1;
                }
            }
            Token::RBrace => {
                let simple = stmt_buf.trim().to_string();
                if !simple.is_empty() {
                    emit_stmt(&simple, indent, &mut out_lines);
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
                if header_needs_colon(&stmt, &block_headers) {
                    emit_header(&stmt, indent, &mut out_lines);
                    indent += 1;
                    indent -= 1;
                } else {
                    emit_stmt(&stmt, indent, &mut out_lines);
                }
            }
            Token::Text(s) => {
                stmt_buf.push_str(s);
            }
        }
        i += 1;
    }
    let tail = stmt_buf.trim();
    if !tail.is_empty() {
        if header_needs_colon(tail, &block_headers) {
            emit_header(tail, indent, &mut out_lines);
        } else {
            emit_stmt(tail, indent, &mut out_lines);
        }
    }
    out_lines.join("\n") + "\n"
}
