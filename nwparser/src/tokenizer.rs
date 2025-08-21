//! Tokenizer for curly-brace/semicolon Python-like language

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    LBrace,
    RBrace,
    Semicolon,
    Text(String),
}

/// Remove line comments (// and #), ignoring those inside quotes (naively)
pub fn strip_line_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\'' && !in_double {
            in_single = !in_single;
        } else if ch == '"' && !in_single {
            in_double = !in_double;
        }
        if !in_single && !in_double {
            if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                break;
            }
            if ch == '#' {
                break;
            }
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// Tokenize source code into tokens: '{', '}', ';', or text chunks
pub fn tokenize(source: &str) -> Vec<Token> {
    let src = source.replace("\r\n", "\n").replace("\r", "\n");
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;

    fn flush_buf(buf: &mut String, tokens: &mut Vec<Token>) {
        if !buf.is_empty() {
            tokens.push(Token::Text(buf.clone()));
            buf.clear();
        }
    }

    for line in src.lines() {
        let clean = strip_line_comment(line);
        for ch in clean.chars().chain(Some(' ')) { // keep newlines as spaces
            if ch == '\'' && !in_double {
                in_single = !in_single;
            } else if ch == '"' && !in_single {
                in_double = !in_double;
            }
            if !in_single && !in_double && (ch == '{' || ch == '}' || ch == ';') {
                flush_buf(&mut buf, &mut tokens);
                match ch {
                    '{' => tokens.push(Token::LBrace),
                    '}' => tokens.push(Token::RBrace),
                    ';' => tokens.push(Token::Semicolon),
                    _ => {}
                }
            } else {
                buf.push(if ch == '\n' { ' ' } else { ch });
            }
        }
    }
    flush_buf(&mut buf, &mut tokens);
    tokens.into_iter().filter(|t| match t { Token::Text(s) => !s.trim().is_empty(), _ => true }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tokenize_basic() {
        let src = "def add(a, b) { return a + b; }";
        let tokens = tokenize(src);
        assert_eq!(tokens, vec![
            Token::Text("def add(a, b) ".to_string()),
            Token::LBrace,
            Token::Text(" return a + b".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ]);
    }
}
