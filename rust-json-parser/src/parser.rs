use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

// JsonParser Struct
pub struct JsonParser {
    tokens: Vec<Token>,
    position: usize,
}

// JsonParser implementation
impl JsonParser {
    pub fn new(input: &str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;

        Ok(JsonParser {
            tokens,
            position: 0,
        })
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            None
        } else {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        }
    }

    // Peek at current token and check if it matches expected variant
    // Uses dicreiminant comparison - matches the variant shape, ignores inner data
    // Returns falso if at end. Does Not advance position
    fn check(&self, expected: &Token) -> bool {
        self.tokens
            .get(self.position)
            .map(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
            .unwrap_or(false)
    }

    // Looks at the current token and routes to the right parser
    // Primitives -> handle directly
    // [ -> call parse_array() (recursive dispatch point)
    fn parse_value(&mut self) -> Result<JsonValue> {
        if self.is_at_end() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.position,
            });
        }

        // Clone current token to decide what to do
        // We clone so we can call &mut self methods (advance, parse_array)
        // without a borrow conflict
        match self.tokens[self.position].clone() {
            // THE RECURSION ENTRY POINT
            // parse_array will call parse_value for each element,
            // which might hit this arm again for nested arrays
            Token::LeftBracket => self.parse_array(),

            // Primitives: data already captured by the clone - just advance and wrap
            Token::String(s) => {
                self.advance();
                Ok(JsonValue::String(s))
            }
            Token::Number(n) => {
                self.advance();
                Ok(JsonValue::Number(n))
            }
            Token::Boolean(b) => {
                self.advance();
                Ok(JsonValue::Boolean(b))
            }
            Token::Null => {
                self.advance();
                Ok(JsonValue::Null)
            }
            token => Err(JsonError::UnexpectedToken {
                expected: "JSON value".to_string(),
                found: format!("{:?}", token),
                position: self.position,
            }),
        }
    }

    // Parses a JSON array: [ value, value, ...]
    // Called by parse_value() when it sees LeftBracket
    // Calls parse_value() for each element -> mutual recursion
    fn parse_array(&mut self) -> Result<JsonValue> {
        self.advance(); // consume opening [
        let mut elements: Vec<JsonValue> = Vec::new();

        // Empty array [] - nothing to collect
        if self.check(&Token::RightBracket) {
            self.advance(); // consume ]
            return Ok(JsonValue::Array(elements));
        }

        // Main collection loop
        loop {
            // Parse next element - THIS is the recursive call
            // If the element is itself an array, parse_value() will
            // call parse_array() again, creating a new stack frame.
            let value = self.parse_value()?;
            elements.push(value); // Vec takes ownership of the parsed value

            if self.check(&Token::Comma) {
                self.advance(); // consume

                // Trailing comma guard: [1,2,] is invalid JSON
                if self.check(&Token::RightBracket) {
                    return Err(JsonError::UnexpectedToken {
                        expected: "JSON value".to_string(),
                        found: "]".to_string(),
                        position: self.position,
                    });
                }
                // valid comma - loop back and parse the next element
            } else if self.check(&Token::RightBracket) {
                self.advance(); // consume ]
                break; // array complete
            } else if self.is_at_end() {
                // Ran out of tokens before seeing ] - unclosed array
                return Err(JsonError::UnexpectedEndOfInput {
                    expected: "] or ,".to_string(),
                    position: self.position,
                });
            } else {
                // Something unexpected - not a comma or ]
                return Err(JsonError::UnexpectedToken {
                    expected: "] or ,".to_string(),
                    found: format!("{:?}", self.tokens[self.position]),
                    position: self.position,
                });
            }
        }
        Ok(JsonValue::Array(elements))
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        // parse_value() now does all the work, including array dispatch
        self.parse_value()
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue> {
    // JsonParser::new tokenizes; parser.parse() recursively handles everything
    let mut parser = JsonParser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;
    type Result<T> = std::result::Result<T, JsonError>;
    #[test]
    fn test_parse_string() -> Result<()> {
        let result = parse_json(r#""hello world""#)?;
        assert_eq!(result, JsonValue::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_number() -> Result<()> {
        let result = parse_json("42.5")?;
        assert_eq!(result, JsonValue::Number(42.5));

        let result = parse_json("0")?;
        assert_eq!(result, JsonValue::Number(0.0));

        let result = parse_json("-10")?;
        assert_eq!(result, JsonValue::Number(-10.0));
        Ok(())
    }

    #[test]
    fn test_parse_boolean() -> Result<()> {
        let result = parse_json("true")?;
        assert_eq!(result, JsonValue::Boolean(true));

        let result = parse_json("false")?;
        assert_eq!(result, JsonValue::Boolean(false));
        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<()> {
        let result = parse_json("null")?;
        assert_eq!(result, JsonValue::Null);
        Ok(())
    }

    #[test]
    fn test_parse_error_empty() {
        let result = parse_json("");
        assert!(result.is_err());
        match result {
            Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                assert_eq!(expected, "JSON value");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedEndOfInput error"),
        }
    }

    #[test]
    fn test_parse_error_invalid_token() {
        let result = parse_json("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let result = parse_json(" 42 ")?;
        assert_eq!(result, JsonValue::Number(42.0));

        let result = parse_json("\n\ttrue\n")?;
        assert_eq!(result, JsonValue::Boolean(true));
        Ok(())
    }

    #[test]
    fn test_result_pattern_matching() {
        let result = parse_json("42");
        match result {
            Ok(JsonValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected successful number parse"),
        }

        let result = parse_json("@invalid@");
        match result {
            Err(JsonError::UnexpectedToken { .. }) => {}
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
    #[test]
    fn test_parse_string_with_newline() {
        let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
        let result = parser.parse().unwrap();
        assert_eq!(result, JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let mut parser = JsonParser::new(r#""\u0048\u0069""#).unwrap();
        let result = parser.parse().unwrap();
        assert_eq!(result, JsonValue::String("Hi".to_string()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let mut parser = JsonParser::new(r#""tab\there\nnewline""#).unwrap();
        let result = parser.parse().unwrap();
        assert_eq!(result, JsonValue::String("tab\there\nnewline".to_string()));
    }
    #[cfg(test)]
    mod array_tests {
        use super::*;

        #[test]
        fn test_parse_empty_array() {
            let value = parse_json("[]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![]));
        }

        #[test]
        fn test_parse_array_single() {
            let value = parse_json("[1]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        }

        #[test]
        fn test_parse_array_multiple() {
            let value = parse_json("[1, 2, 3]").unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }

        #[test]
        fn test_parse_array_mixed_types() {
            let value = parse_json(r#"[1, "two", true, null]"#).unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::String("two".to_string()));
            assert_eq!(arr[2], JsonValue::Boolean(true));
            assert_eq!(arr[3], JsonValue::Null);
        }
    }
}
