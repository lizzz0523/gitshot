/// A token produced by line tokenization: identifier run, whitespace run, or single punctuation.
pub struct Token {
    /// Character offset range within the original string.
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub is_whitespace: bool,
}

/// Tokenize a line of code into identifiers, whitespace runs, and individual punctuation chars.
pub fn tokens(s: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if is_identifier_char(c) {
            let start = i;
            while i < chars.len() && is_identifier_char(chars[i]) {
                i += 1;
            }
            result.push(Token {
                start,
                end: i,
                text: chars[start..i].iter().collect(),
                is_whitespace: false,
            });
        } else if c == ' ' || c == '\t' {
            let start = i;
            while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                i += 1;
            }
            result.push(Token {
                start,
                end: i,
                text: chars[start..i].iter().collect(),
                is_whitespace: true,
            });
        } else {
            // Each punctuation character is its own token.
            result.push(Token {
                start: i,
                end: i + 1,
                text: c.to_string(),
                is_whitespace: false,
            });
            i += 1;
        }
    }

    result
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
