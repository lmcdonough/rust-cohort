use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer, tokenize};
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

    pub fn parse(&mut self) -> Result<JsonValue> {
        if self.is_at_end() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "a JSON value".to_string(),
                position: 0,
            });
        }

        // FIX 1: This match block was OUTSIDE the parse() function.
        //        It must be inside parse() — it's the return value.
        match self.advance() {
            Some(Token::String(s)) => Ok(JsonValue::String(s)),
            Some(Token::Number(n)) => Ok(JsonValue::Number(n)),
            Some(Token::Boolean(b)) => Ok(JsonValue::Boolean(b)),
            Some(Token::Null) => Ok(JsonValue::Null),
            // FIX 2: Missing closing paren — was `Some(token =>`
            //        Correct syntax: `Some(token) =>`
            Some(token) => Err(JsonError::UnexpectedToken {
                expected: "a JSON value".to_string(),
                found: format!("{:?}", token),
                // FIX 3: Typo — was `positino`, corrected to `position`
                position: self.position - 1,
            }),
            // FIX 4: Was `None = Err(...)` — missing `>` in the arrow
            //        Correct syntax: `None => Err(...)`
            // FIX 5: Was `UnexpectedEndOFInput` — capital "OF" typo
            //        Correct: `UnexpectedEndOfInput`
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "a JSON value".to_string(),
                position: self.position,
            }),
        }
    } // FIX 6: parse() now properly closes here, after the match block
}

pub fn parse_json(input: &str) -> Result<JsonValue> {
    let tokens = tokenize(input)?;

    if tokens.is_empty() {
        return Err(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: 0,
        });
    }

    match &tokens[0] {
        Token::String(s) => Ok(JsonValue::String(s.clone())),
        Token::Number(n) => Ok(JsonValue::Number(*n)),
        Token::Boolean(b) => Ok(JsonValue::Boolean(*b)),
        Token::Null => Ok(JsonValue::Null),
        _ => Err(JsonError::UnexpectedToken {
            expected: "primitive JSON value".to_string(),
            found: format!("{:?}", tokens[0]),
            position: 0,
        }),
    }
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
}
