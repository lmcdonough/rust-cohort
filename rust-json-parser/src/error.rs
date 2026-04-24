use std::fmt;

/// Errors produced by the JSON parser.
///
/// Each variant captures enough context to locate and understand
/// the failure — what was expected, what was found, and where.
///
/// # Examples
///
/// ```
/// use rust_json_parser::{parse_json, JsonError};
///
/// // invalid input triggers a parse error
/// let err = parse_json("{bad}").unwrap_err();
/// // match the specific variant to react to different failure modes
/// assert!(matches!(err, JsonError::UnexpectedToken { .. }));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// Encountered a token that does not match the grammar at this position.
    ///
    /// Occurs when the parser sees a token that is valid on its own but
    /// not allowed in the current context — e.g. a `]` inside an object,
    /// or a stray `,` with no preceding value.
    ///
    /// Fields:
    /// - `expected`: human-readable description of what the parser wanted
    /// - `found`: the actual token text encountered
    /// - `position`: byte offset into the input where the token started
    UnexpectedToken {
        /// Human-readable description of what the parser wanted.
        expected: String,
        /// The actual token text encountered.
        found: String,
        /// Byte offset into the input where the token started.
        position: usize,
    },

    /// Input ended before a complete JSON value could be parsed.
    ///
    /// Triggered by truncated input like `{"key":` (missing value) or
    /// `[1, 2,` (missing trailing element). The parser was mid-production
    /// and ran out of characters.
    ///
    /// Fields:
    /// - `expected`: what the parser was waiting for (e.g. `"closing quote"`)
    /// - `position`: byte offset where input ran out
    UnexpectedEndOfInput {
        /// What the parser was waiting for (e.g. `"closing quote"`).
        expected: String,
        /// Byte offset where input ran out.
        position: usize,
    },

    /// A number literal failed to parse as a valid `f64`.
    ///
    /// Occurs for malformed numbers like `12.34.56`, `1e`, or values
    /// outside `f64` range. The tokenizer accepted the digit sequence
    /// but conversion to a float failed.
    ///
    /// Fields:
    /// - `value`: the offending number string
    /// - `position`: byte offset where the number started
    InvalidNumber {
        /// The offending number string.
        value: String,
        /// Byte offset where the number started.
        position: usize,
    },

    /// A string contained a backslash followed by a character that is
    /// not a recognized JSON escape.
    ///
    /// JSON permits `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`, and
    /// `\uXXXX`. Anything else — e.g. `"\q"` or `"\x"` — produces this
    /// error at the position of the offending character.
    ///
    /// Fields:
    /// - `char`: the character that followed the backslash
    /// - `position`: byte offset of that character in the input
    InvalidEscape {
        /// The character that followed the backslash.
        char: char,
        /// Byte offset of that character in the input.
        position: usize,
    },

    /// A `\u` escape was not followed by four valid hexadecimal digits,
    /// or the resulting code point was not a valid Unicode scalar.
    ///
    /// Covers cases like `"\u12"` (too short), `"\uZZZZ"` (non-hex),
    /// and unpaired surrogates that cannot be converted to a `char`.
    ///
    /// Fields:
    /// - `sequence`: the raw escape text as seen in the source
    /// - `position`: byte offset where the `\u` began
    InvalidUnicode {
        /// The raw escape text as seen in the source.
        sequence: String,
        /// Byte offset where the `\u` began.
        position: usize,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token at position {}: expected {}, found {}",
                    position, expected, found
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(
                    f,
                    "Unexpected end of input at position {}: expected {}",
                    position, expected
                )
            }
            JsonError::InvalidNumber { value, position } => {
                write!(f, "Invalid number '{}' at position {}", value, position)
            }
            JsonError::InvalidEscape { char, position } => {
                write!(f, "Invalid escape '\\{}' at position {}", char, position)
            }
            JsonError::InvalidUnicode { sequence, position } => write!(
                f,
                "Invalid unicode escape '{}' at position {}",
                sequence, position
            ),
        }
    }
}

impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "@".to_string(),
            position: 5,
        };
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }

    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".to_string(),
            found: "@".to_string(),
            position: 0,
        };
        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }

    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "x".to_string(),
            position: 3,
        };
        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".to_string(),
            position: 10,
        };
        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".to_string(),
            position: 0,
        };
        let _ = format!("{:?}", token_error);
        let _ = format!("{:?}", eof_error);
        let _ = format!("{:?}", num_error);
    }
}
