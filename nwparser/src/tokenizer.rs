//! Tokenizer for curly-brace/semicolon Python-like language

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    LBrace,
    RBrace,
    Semicolon,
    Text(String),
    Comment(String),
}

/// Tokenize source code into tokens: '{', '}', ';', text chunks, or comments
pub fn tokenize(source: &str) -> Vec<Token> {
    let src = source.replace("\r\n", "\n").replace("\r", "\n");
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut in_multiline_comment = false;
    let mut multiline_comment = String::new();

    fn flush_buf(buf: &mut String, tokens: &mut Vec<Token>) {
        if !buf.is_empty() {
            tokens.push(Token::Text(buf.clone()));
            buf.clear();
        }
    }

    let lines = src.lines();
    for line in lines {
        let mut comment_start = None;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut in_single = false;
        let mut in_double = false;
        while i < chars.len() {
            let ch = chars[i];
            if in_multiline_comment {
                if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    multiline_comment.push_str("*/");
                    tokens.push(Token::Comment(multiline_comment.trim().to_string()));
                    multiline_comment.clear();
                    in_multiline_comment = false;
                    i += 2;
                    continue;
                } else {
                    multiline_comment.push(ch);
                    i += 1;
                    continue;
                }
            }
            if ch == '\'' && !in_double {
                in_single = !in_single;
            } else if ch == '"' && !in_single {
                in_double = !in_double;
            }
            if !in_single && !in_double {
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    comment_start = Some(i);
                    break;
                }
                if ch == '#' {
                    comment_start = Some(i);
                    break;
                }
                if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                    in_multiline_comment = true;
                    multiline_comment.push_str("/*");
                    i += 2;
                    continue;
                }
                if ch == '{' {
                    flush_buf(&mut buf, &mut tokens);
                    tokens.push(Token::LBrace);
                    i += 1;
                    continue;
                }
                if ch == '}' {
                    flush_buf(&mut buf, &mut tokens);
                    tokens.push(Token::RBrace);
                    i += 1;
                    continue;
                }
                if ch == ';' {
                    flush_buf(&mut buf, &mut tokens);
                    tokens.push(Token::Semicolon);
                    i += 1;
                    continue;
                }
            }
            buf.push(ch);
            i += 1;
        }
        flush_buf(&mut buf, &mut tokens);
        if let Some(idx) = comment_start {
            let comment = &line[idx..];
            tokens.push(Token::Comment(comment.trim().to_string()));
        }
        // If still in multiline comment, add newline
        if in_multiline_comment {
            multiline_comment.push('\n');
        }
    }
    flush_buf(&mut buf, &mut tokens);
    tokens
        .into_iter()
        .filter(|t| match t {
            Token::Text(s) => !s.trim().is_empty(),
            Token::Comment(s) => !s.trim().is_empty(),
            _ => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tokenize_basic() {
        let src = "def add(a, b) { return a + b; }";
        let tokens = tokenize(src);
        assert_eq!(
            tokens,
            vec![
                Token::Text("def add(a, b) ".to_string()),
                Token::LBrace,
                Token::Text(" return a + b".to_string()),
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }
}
