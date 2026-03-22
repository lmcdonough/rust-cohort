use crate::error::JsonError;
use crate::tokenizer::{Token, tokenize};
use crate::value::JsonValue;

type Result<T> = std::result::Result<T, JsonError>;

pub fn parse_json(input: &str) -> Result<JsonValue> {
    // tokenize the input
    // ? means: if tokenize returns Err, immediately return that Err from parse_json
    // no mut needed: we're not modifying the vector, just reading from it
    let tokens = tokenize(input)?;

    // check for empty input
    // tokenize returns a Vec, not an iterator so use .is_empty() not .peek()
    // .peek() is for iterators (like chars.peek() in the tokenizer function)
    if tokens.is_empty() {
        return Err(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: 0,
        });
    }

    // Match on the first token and convert to JsonValue
    // &tokens[0] - we borrow the first element (& means "look at it, dont take it")
    match &tokens[0] {
        // Destructure: Token::String(s) pulls the string DATA out of the variant
        // Python analogy: like unpacking - s = token.value
        Token::String(s) => Ok(JsonValue::String(s.clone())),

        // *n dereferences: n is &f64, *n copies the actual number
        Token::Number(n) => Ok(JsonValue::Number(*n)),

        // *b dereferences: same pattern, copies the bool
        Token::Boolean(b) => Ok(JsonValue::Boolean(*b)),

        // Null carries no data, so no destructuring is needed
        Token::Null => Ok(JsonValue::Null),

        // Anything else is an error - format the token for the error message
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

    #[test]
    fn test_parse_string() {
        let result = parse_json(r#""hello world""#).unwrap();
        assert_eq!(result, JsonValue::String("hello world".to_string()));
    }

    #[test]
    fn test_parse_number() {
        let result = parse_json("42.5").unwrap();
        assert_eq!(result, JsonValue::Number(42.5));

        let result = parse_json("0").unwrap();
        assert_eq!(result, JsonValue::Number(0.0));

        let result = parse_json("-10").unwrap();
        assert_eq!(result, JsonValue::Number(-10.0));
    }

    #[test]
    fn test_parse_boolean() {
        let result = parse_json("true").unwrap();
        assert_eq!(result, JsonValue::Boolean(true));

        let result = parse_json("false").unwrap();
        assert_eq!(result, JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let result = parse_json("null").unwrap();
        assert_eq!(result, JsonValue::Null);
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
    fn test_parse_with_whitespace() {
        let result = parse_json(" 42 ").unwrap();
        assert_eq!(result, JsonValue::Number(42.0));

        let result = parse_json("\n\ttrue\n").unwrap();
        assert_eq!(result, JsonValue::Boolean(true));
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
}
