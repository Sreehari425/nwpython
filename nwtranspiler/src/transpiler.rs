//! Transpiler: convert tokens to Python code
use nwparser::tokenizer::Token;

pub fn transpile(tokens: &[Token]) -> String {
    let mut out_lines = Vec::new();
    let mut indent = 0;
    let mut stmt_buf = String::new();
    let block_headers = ["if ", "elif ", "else", "def ", "while ", "for "];
    let mut block_stack: Vec<&str> = Vec::new(); // Track what kind of blocks we're in

    fn header_needs_colon(s: &str, block_headers: &[&str]) -> bool {
        let s = s.trim();
        block_headers.iter().any(|h| s.starts_with(h))
    }

    fn is_function_header(s: &str) -> bool {
        s.trim().starts_with("def ")
    }

    let re_print_post_inc =
        regex::Regex::new(r"print\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\+\+\s*\)").unwrap();
    let re_print_post_dec =
        regex::Regex::new(r"print\s*\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*--\s*\)").unwrap();
    let re_print_pre_inc =
        regex::Regex::new(r"print\s*\(\s*\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)").unwrap();
    let re_print_pre_dec =
        regex::Regex::new(r"print\s*\(\s*--\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)").unwrap();
    let re_ret_pre_inc = regex::Regex::new(r"return\s*\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let re_ret_pre_dec = regex::Regex::new(r"return\s*--\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let re_pre_inc = regex::Regex::new(r"\+\+\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let re_pre_dec = regex::Regex::new(r"--\s*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let re_inc = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\+\+").unwrap();
    let re_dec = regex::Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*--").unwrap();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Comment(c) => {
                let c = c.trim_start_matches(['/', '#', ' ']);
                if c.starts_with("*") {
                    // Multi-line comment: /* ... */
                    let content = c
                        .trim_start_matches('*')
                        .trim_start_matches('/')
                        .trim_end_matches("*/")
                        .trim();
                    out_lines.push(format!("{}\"\"\"{}\"\"\"", "    ".repeat(indent), content));
                } else {
                    // Single-line comment
                    out_lines.push(format!("{}# {}", "    ".repeat(indent), c));
                }
            }
            Token::Text(s) => {
                let mut line = s.to_string();

                let mut handled = false;
                if re_print_post_inc.is_match(&line) {
                    let var = re_print_post_inc
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str();
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_post_dec.is_match(&line) {
                    let var = re_print_post_dec
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str();
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_pre_inc.is_match(&line) {
                    let var = re_print_pre_inc
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str();
                    out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    handled = true;
                }
                if re_print_pre_dec.is_match(&line) {
                    let var = re_print_pre_dec
                        .captures(&line)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str();
                    out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                    out_lines.push(format!("{}print({})", "    ".repeat(indent), var));
                    handled = true;
                }
                if handled {
                    // Don't process further
                } else {
                    // Handle return x++ and return x-- (post-increment/decrement)

                    let mut handled_ret = false;
                    if re_ret_pre_inc.is_match(&line) {
                        let var = re_ret_pre_inc
                            .captures(&line)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str();
                        stmt_buf.push_str("");
                        out_lines.push(format!("{}{} += 1", "    ".repeat(indent), var));
                        out_lines.push(format!("{}return {}", "    ".repeat(indent), var));
                        handled_ret = true;
                    }
                    if re_ret_pre_dec.is_match(&line) {
                        let var = re_ret_pre_dec
                            .captures(&line)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str();
                        stmt_buf.push_str("");
                        out_lines.push(format!("{}{} -= 1", "    ".repeat(indent), var));
                        out_lines.push(format!("{}return {}", "    ".repeat(indent), var));
                        handled_ret = true;
                    }
                    if handled_ret {
                        // Don't process further
                    } else {
                        // Handle ++x and --x (standalone)

                        line = re_pre_inc.replace_all(&line, "$1 += 1").to_string();
                        line = re_pre_dec.replace_all(&line, "$1 -= 1").to_string();
                        // Handle x++ and x-- (standalone)

                        line = re_inc.replace_all(&line, "$1 += 1").to_string();
                        line = re_dec.replace_all(&line, "$1 -= 1").to_string();

                        // Fix dictionary iteration: .values -> .items() for key,value pairs
                        if line.contains("for ")
                            && line.contains(" in ")
                            && line.contains(".values")
                        {
                            line = line.replace(".values", ".items()");
                        }

                        stmt_buf.push_str(&line);
                    }
                }
            }
            Token::LBrace => {
                let header = stmt_buf.trim().to_string();

                // Check if this is a dictionary literal by analyzing the context
                let is_dictionary = if header.is_empty() {
                    // Empty header could be a standalone dictionary
                    true
                } else if header.contains('=') {
                    // Contains assignment, likely: var = {
                    let after_equals = header.split('=').last().unwrap_or("").trim();
                    after_equals.is_empty() || !header_needs_colon(&header, &block_headers)
                } else {
                    // Check if this is a control structure that needs a colon
                    !header_needs_colon(&header, &block_headers)
                };

                if is_dictionary {
                    // This is a dictionary literal, keep the brace as part of the statement
                    stmt_buf.push('{');
                } else {
                    // This is a code block
                    stmt_buf.clear();
                    let mut line = header.clone();
                    if !line.ends_with(':') {
                        line.push(':');
                    }
                    out_lines.push(format!("{}{}", "    ".repeat(indent), line));

                    // Track what kind of block we're entering
                    if is_function_header(&header) {
                        block_stack.push("function");
                    } else {
                        block_stack.push("control");
                    }

                    indent += 1;
                }
            }
            Token::RBrace => {
                // Check if we're in a dictionary context by looking at the statement buffer
                let current_stmt = stmt_buf.trim();
                let in_dictionary = current_stmt.contains('{')
                    || current_stmt.contains(':')
                        && !current_stmt.starts_with("if ")
                        && !current_stmt.starts_with("elif ")
                        && !current_stmt.starts_with("else")
                        && !current_stmt.starts_with("def ")
                        && !current_stmt.starts_with("while ")
                        && !current_stmt.starts_with("for ");

                if in_dictionary {
                    // This is closing a dictionary literal
                    stmt_buf.push('}');
                } else {
                    // This is closing a code block
                    let simple = stmt_buf.trim().to_string();
                    let in_function_context = block_stack.contains(&"function");

                    if !simple.is_empty() {
                        let statements: Vec<&str> = simple.split(';').collect();
                        for (idx, stmt) in statements.iter().enumerate() {
                            let s = stmt.trim();
                            if !s.is_empty() {
                                // Handle 'let' keyword conversion to standard variable assignment

                                let s = if let Some(stripped) = s.strip_prefix("let ") {
                                    stripped
                                } else {
                                    s
                                };
                                // Check if this is the last statement and we should auto-return
                                let should_auto_return = in_function_context
                                    && idx == statements.len() - 1
                                    && !s.starts_with("return ")
                                    && !s.trim().is_empty()
                                    && !s.starts_with("print(")
                                    && !s.starts_with("print ");

                                if should_auto_return {
                                    // Auto-return: add return prefix to the last expression
                                    out_lines.push(format!(
                                        "{}return {}",
                                        "    ".repeat(indent),
                                        s
                                    ));
                                } else {
                                    out_lines.push(format!("{}{}", "    ".repeat(indent), s));
                                }
                            }
                        }
                        stmt_buf.clear();
                    }

                    if indent > 0 {
                        indent -= 1;
                        block_stack.pop();
                    } else {
                        out_lines.push("# ERROR: Too many '}'".to_string());
                    }
                }
            }
            Token::Semicolon => {
                let stmt = stmt_buf.trim().to_string();
                stmt_buf.clear();
                for s in stmt.split(';') {
                    let s = s.trim();
                    if !s.is_empty() {
                        let s = if let Some(stripped) = s.strip_prefix("let ") {
                            stripped
                        } else {
                            s
                        };
                        if header_needs_colon(s, &block_headers) {
                            let mut line = s.to_string();
                            if !line.ends_with(':') {
                                line.push(':');
                            }
                            out_lines.push(format!("{}{}", "    ".repeat(indent), line));

                            // Track what kind of block we're entering
                            if is_function_header(s) {
                                block_stack.push("function");
                            } else {
                                block_stack.push("control");
                            }

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
                let s = if let Some(stripped) = s.strip_prefix("let ") {
                    stripped
                } else {
                    s
                };

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
