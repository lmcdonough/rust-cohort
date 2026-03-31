use crate::error::JsonError;

// Struct definition
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

// Tokenizer implementation
impl Tokenizer {
    // constructor: borrows &str, converts to owned Vec<char>
    // no self parameter because this creates a new instance
    // Called as Tokenizer::new("input"), not instance.new()
    pub fn new(input: &str) -> Self {
        Self {
            // .chars() iterates Unicode chars, .collect() gathers into Vec
            input: input.chars().collect(),
            // Start reading from the beginning
            position: 0,
        }
    }

    // Read current char without moving forward
    // &self = read-only access, position stays the same
    // Return None if we've consumed all input
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    // read current char and move forward by one
    // &mut self = se need to modify self.position
    // Returns None if we've consumed all input
    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            let ch = self.input[self.position];
            self.position += 1;
            Some(ch)
        }
    }

    // Check if we've consumed all input
    // &self = read-only, just comparing two numbers
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    // Reads a quoted JSON string from chars at position and outputs Token::String or a JsonError.
    fn tokenize_string(&mut self) -> Result<Token, JsonError> {
        let start = self.position;
        let mut s = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    break;
                }

                // Adds escape sequence handling
                Some('\\') => {
                    self.advance(); // consume the backslash
                    match self.advance() {
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some('/') => s.push('/'),
                        Some('b') => s.push('\u{0008}'), // backspace
                        Some('f') => s.push('\u{000C}'), // form feed
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        // Step 4 will add Some('u') here for unicode
                        Some('u') => {
                            let mut hex_str = String::new();
                            for _ in 0..4 {
                                match self.advance() {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        hex_str.push(c);
                                    }
                                    _ => {
                                        return Err(JsonError::InvalidUnicode {
                                            sequence: hex_str,
                                            position: self.position,
                                        });
                                    }
                                }
                            }
                            let code_point = u32::from_str_radix(&hex_str, 16).map_err(|_| {
                                JsonError::InvalidUnicode {
                                    sequence: hex_str.clone(),
                                    position: self.position,
                                }
                            })?;
                            let ch =
                                char::from_u32(code_point).ok_or(JsonError::InvalidUnicode {
                                    sequence: hex_str,
                                    position: self.position,
                                })?;
                            s.push(ch);
                            continue; // skip back to loop top
                        }
                        Some(ch) => {
                            return Err(JsonError::InvalidEscape {
                                char: ch,
                                position: self.position - 1,
                            });
                        }
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "escape character".to_string(),
                                position: self.position,
                            });
                        }
                    }
                }
                Some(ch) => {
                    self.advance();
                    s.push(ch);
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "closing quote".to_string(),
                        position: start,
                    });
                }
            }
        }
        Ok(Token::String(s))
    }

    // Reads a JSON number from chars at position and outputs Token::Number or InvalidNumber.
    fn tokenize_number(&mut self) -> Result<Token, JsonError> {
        let start = self.position;
        let mut num_str = String::new();
        while !self.is_at_end() {
            match self.peek() {
                Some(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                    self.advance();
                    num_str.push(c);
                }
                _ => break,
            }
        }
        let number = num_str
            .parse::<f64>()
            .map_err(|_| JsonError::InvalidNumber {
                value: num_str.clone(),
                position: start,
            })?;
        Ok(Token::Number(number))
    }

    // Reads an alphabetic keyword from chars at position and outputs true/false/null tokens or an error.
    fn tokenize_keyword(&mut self) -> Result<Token, JsonError> {
        let start = self.position;
        let mut keyword = String::new();
        while !self.is_at_end() {
            match self.peek() {
                Some(c) if c.is_alphabetic() => {
                    self.advance();
                    keyword.push(c);
                }
                _ => break,
            }
        }
        match keyword.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err(JsonError::UnexpectedToken {
                expected: "true, false, or null".to_string(),
                found: keyword,
                position: start,
            }),
        }
    }

    // New public tokenize function is clean and short
    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('{') => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                }
                Some('}') => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                }
                Some('[') => {
                    self.advance();
                    tokens.push(Token::LeftBracket);
                }
                Some(']') => {
                    self.advance();
                    tokens.push(Token::RightBracket);
                }
                Some(':') => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                Some(',') => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                Some('"') => {
                    self.advance(); // consume opening quote
                    tokens.push(self.tokenize_string()?);
                }
                Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                    tokens.push(self.tokenize_number()?);
                }
                Some(ch) if ch.is_alphabetic() => {
                    tokens.push(self.tokenize_keyword()?);
                }
                Some(ch) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON token".to_string(),
                        found: ch.to_string(),
                        position: self.position,
                    });
                }
                None => break,
            }
        }
        Ok(tokens)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    Tokenizer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    // Verifies tokenize with "{}" input returns left/right brace tokens; output is Result<()>.
    fn test_empty_braces() -> Result<()> {
        let tokens = tokenize("{}")?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }

    #[test]
    // Verifies tokenize with a quoted string input returns one Token::String; output is Result<()>.
    fn test_simple_string() -> Result<()> {
        let tokens = tokenize(r#""hello""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize with numeric input returns one Token::Number; output is Result<()>.
    fn test_number() -> Result<()> {
        let tokens = tokenize("42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    // Verifies tokenize with a spaced quoted string input returns one Token::String; output is Result<()>.
    fn test_tokenize_string() -> Result<()> {
        let tokens = tokenize(r#""hello world""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize with boolean/null input returns matching keyword tokens; output is Result<()>.
    fn test_boolean_and_null() -> Result<()> {
        let tokens = tokenize("true false null")?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
    // Verifies tokenize with a simple object input returns expected object token sequence; output is Result<()>.
    fn test_simple_object() -> Result<()> {
        let tokens = tokenize(r#"{"name": "Alice"}"#)?;
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
        Ok(())
    }

    #[test]
    // Verifies tokenize with multi-field object input includes expected string/number/boolean/comma tokens; output is Result<()>.
    fn test_multiple_values() -> Result<()> {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#)?;
        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
        Ok(())
    }

    #[test]
    // Verifies tokenize with empty quoted string input returns Token::String(""); output is Result<()>.
    fn test_empty_string() -> Result<()> {
        let tokens = tokenize(r#""""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize treats JSON-like punctuation inside quoted input as string content; output is Result<()>.
    fn test_string_containing_json_special_chars() -> Result<()> {
        let tokens = tokenize(r#""{key: value}""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize keeps keyword-like words inside quoted input as one string token; output is Result<()>.
    fn test_string_with_keyword_like_content() -> Result<()> {
        let tokens = tokenize(r#""not true or false""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize keeps number-like text inside quoted input as one string token; output is Result<()>.
    fn test_string_with_number_like_content() -> Result<()> {
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    // Verifies tokenize with negative number input returns one Token::Number(-42.0); output is Result<()>.
    fn test_negative_number() -> Result<()> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    // Verifies tokenize with decimal input returns one Token::Number(0.5); output is Result<()>.
    fn test_decimal_number() -> Result<()> {
        let tokens = tokenize("0.5")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    // Verifies tokenize with leading-decimal input ".5" returns an error; output is ().
    fn test_leading_decimal_not_a_number() {
        let result = tokenize(".5");
        assert!(result.is_err());
    }

    #[test]
    // Verifies tokenize reports invalid-keyword error at the keyword start index; input is "   xyz", output is ().
    fn test_invalid_keyword_error_position_points_to_start() {
        let input = "   xyz";
        let result = tokenize(input);
        assert!(result.is_err());
        if let Err(JsonError::UnexpectedToken { position, .. }) = result {
            assert_eq!(
                position, 3,
                "error position should point to the start of 'xyz' (index 3), not past it"
            );
        } else {
            panic!("expected UnexpectedToken error");
        }
    }
}
