use std::collections::HashMap;

use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

pub struct JsonParser {
    tokens: Vec<Token>,
    position: usize,
}

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

    fn check(&self, expected: &Token) -> bool {
        self.tokens
            .get(self.position)
            .map(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
            .unwrap_or(false)
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        if self.is_at_end() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.position,
            });
        }

        match self.tokens[self.position].clone() {
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
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

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.advance();
        let mut elements: Vec<JsonValue> = Vec::new();

        if self.check(&Token::RightBracket) {
            self.advance();
            return Ok(JsonValue::Array(elements));
        }

        loop {
            let value = self.parse_value()?;
            elements.push(value);

            if self.check(&Token::Comma) {
                self.advance();

                if self.check(&Token::RightBracket) {
                    return Err(JsonError::UnexpectedToken {
                        expected: "JSON value".to_string(),
                        found: "]".to_string(),
                        position: self.position,
                    });
                }
            } else if self.check(&Token::RightBracket) {
                self.advance();
                break;
            } else if self.is_at_end() {
                return Err(JsonError::UnexpectedEndOfInput {
                    expected: "] or ,".to_string(),
                    position: self.position,
                });
            } else {
                return Err(JsonError::UnexpectedToken {
                    expected: "] or ,".to_string(),
                    found: format!("{:?}", self.tokens[self.position]),
                    position: self.position,
                });
            }
        }
        Ok(JsonValue::Array(elements))
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.advance();
        let mut map = HashMap::new();

        if self.check(&Token::RightBrace) {
            self.advance();
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.advance() {
                Some(Token::String(s)) => s,
                Some(token) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "string key".to_string(),
                        found: format!("{:?}", token),
                        position: self.position - 1,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "string key".to_string(),
                        position: self.position,
                    });
                }
            };

            match self.advance() {
                Some(Token::Colon) => {}
                Some(token) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: ":".to_string(),
                        found: format!("{:?}", token),
                        position: self.position - 1,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: ":".to_string(),
                        position: self.position,
                    });
                }
            }

            let value = self.parse_value()?;
            map.insert(key, value);

            match self.advance() {
                Some(Token::RightBrace) => break,
                Some(Token::Comma) => {
                    if self.check(&Token::RightBrace) {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string key".to_string(),
                            found: "}".to_string(),
                            position: self.position,
                        });
                    }
                }
                Some(token) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "} or ,".to_string(),
                        found: format!("{:?}", token),
                        position: self.position - 1,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "} or ,".to_string(),
                        position: self.position,
                    });
                }
            }
        }
        Ok(JsonValue::Object(map))
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        self.parse_value()
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue> {
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

    #[cfg(test)]
    mod object_tests {
        use super::*;
        use std::collections::HashMap;

        #[test]
        fn test_parse_empty_object() {
            let value = parse_json("{}").unwrap();
            assert_eq!(value, JsonValue::Object(HashMap::new()));
        }

        #[test]
        fn test_parse_object_single_key() {
            let value = parse_json(r#"{"key": "value"}"#).unwrap();
            let mut expected = HashMap::new();
            expected.insert("key".to_string(), JsonValue::String("value".to_string()));
            assert_eq!(value, JsonValue::Object(expected));
        }

        #[test]
        fn test_parse_object_multiple_keys() {
            let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
            if let JsonValue::Object(obj) = value {
                assert_eq!(
                    obj.get("name"),
                    Some(&JsonValue::String("Alice".to_string()))
                );
                assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_nested_object() {
            let value = parse_json(r#"{"outer": {"inner": 1}}"#).unwrap();
            if let JsonValue::Object(outer) = value {
                if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                    assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
                } else {
                    panic!("Expected nested object");
                }
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_array_in_object() {
            let value = parse_json(r#"{"items": [1, 2, 3]}"#).unwrap();
            if let JsonValue::Object(obj) = value {
                if let Some(JsonValue::Array(arr)) = obj.get("items") {
                    assert_eq!(arr.len(), 3);
                } else {
                    panic!("Expected array");
                }
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_object_in_array() {
            let value = parse_json(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
            if let JsonValue::Array(arr) = value {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected array");
            }
        }
    }

    #[cfg(test)]
    mod error_tests {
        use super::*;

        #[test]
        fn test_error_unclosed_array() {
            let result = parse_json("[1, 2");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_unclosed_object() {
            let result = parse_json(r#"{"key": 1"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_array() {
            let result = parse_json("[1, 2,]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_object() {
            let result = parse_json(r#"{"a": 1,}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_colon() {
            let result = parse_json(r#"{"key" 1}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_invalid_key() {
            let result = parse_json(r#"{123: "value"}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_array() {
            let result = parse_json("[1 2 3]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_object() {
            let result = parse_json(r#"{"a": 1 "b": 2}"#);
            assert!(result.is_err());
        }
    }
}
