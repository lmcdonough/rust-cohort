use std::iter::Peekable;
use std::str::Chars;

use crate::error::JsonError;

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

fn tokenize_string(chars: &mut Peekable<Chars>, position: &mut usize) -> Result<Token, JsonError> {
    let start = *position;
    chars.next();
    *position += 1;
    let mut s = String::new();
    loop {
        match chars.next() {
            Some('"') => {
                *position += 1;
                break;
            }
            Some(ch) => {
                s.push(ch);
                *position += ch.len_utf8();
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

fn tokenize_number(chars: &mut Peekable<Chars>, position: &mut usize) -> Result<Token, JsonError> {
    let start = *position;
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() || ch == '.' || ch == '-' {
            num_str.push(ch);
            chars.next();
            *position += 1;
        } else {
            break;
        }
    }
    let parsed = num_str
        .parse::<f64>()
        .map_err(|_| JsonError::InvalidNumber {
            value: num_str.clone(),
            position: start,
        })?;
    Ok(Token::Number(parsed))
}

fn tokenize_keyword(chars: &mut Peekable<Chars>, position: &mut usize) -> Result<Token, JsonError> {
    let start = *position;
    let mut word = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_alphabetic() {
            word.push(ch);
            chars.next();
            *position += 1;
        } else {
            break;
        }
    }
    match word.as_str() {
        "true" => Ok(Token::Boolean(true)),
        "false" => Ok(Token::Boolean(false)),
        "null" => Ok(Token::Null),
        _ => Err(JsonError::UnexpectedToken {
            expected: "true, false, or null".to_string(),
            found: word,
            position: start,
        }),
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut position = 0;

    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
                position += 1;
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
                position += 1;
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
                position += 1;
            }
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
                position += 1;
            }
            '"' => {
                tokens.push(tokenize_string(&mut chars, &mut position)?);
            }
            '0'..='9' | '-' => {
                tokens.push(tokenize_number(&mut chars, &mut position)?);
            }
            ch if ch.is_alphabetic() => {
                tokens.push(tokenize_keyword(&mut chars, &mut position)?);
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
                position += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
                position += 1;
            }
            ' ' | '\n' | '\t' | '\r' => {
                chars.next();
                position += 1;
            }
            _ => {
                return Err(JsonError::UnexpectedToken {
                    expected: "valid JSON token".to_string(),
                    found: ch.to_string(),
                    position,
                });
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;

    type Result<T> = std::result::Result<T, JsonError>;

    #[test]
    fn test_empty_braces() -> Result<()> {
        let tokens = tokenize("{}")?;
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
        Ok(())
    }

    #[test]
    fn test_simple_string() -> Result<()> {
        let tokens = tokenize(r#""hello""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        Ok(())
    }

    #[test]
    fn test_number() -> Result<()> {
        let tokens = tokenize("42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
        Ok(())
    }

    #[test]
    fn test_tokenize_string() -> Result<()> {
        let tokens = tokenize(r#""hello world""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
        Ok(())
    }

    #[test]
    fn test_boolean_and_null() -> Result<()> {
        let tokens = tokenize("true false null")?;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
        Ok(())
    }

    #[test]
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
    fn test_empty_string() -> Result<()> {
        let tokens = tokenize(r#""""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        let tokens = tokenize(r#""{key: value}""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        let tokens = tokenize(r#""not true or false""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    #[test]
    fn test_negative_number() -> Result<()> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let tokens = tokenize("0.5")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        let result = tokenize(".5");
        assert!(result.is_err());
    }

    #[test]
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
