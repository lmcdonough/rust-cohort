//! # rust_json_parser
//!
//! A JSON parser written in Rust, exposed to Python via PyO3.
//!
//! This crate tokenizes, parses, and serializes JSON data with
//! safe error handling and zero-copy string views where possible.
//!
//! ## Features
//!
//! - Full JSON spec support: null, booleans, numbers, strings, arrays, objects
//! - Descriptive errors with position information
//! - Python bindings via PyO3 for cross-language use
//! - Pretty-printing with configurable indentation
//!
//! ## Quick Start
//!
//! ```
//! use rust_json_parser::{parse_json, JsonValue};
//!
//! let value = parse_json(r#"{"name": "Levi", "year": 2026}"#)?;
//! assert!(matches!(value, JsonValue::Object(_)));
//! # Ok::<(), rust_json_parser::JsonError>(())
//! ```
//!
//! ## Error Handling
//!
//! All parsing functions return [`Result<JsonValue, JsonError>`]. See
//! [`JsonError`] for the full list of error variants.

// #! = attribute applied to the ENTIRE crate (not next item)
// warn = emit warning (not error) — lets you iterate without blocking builds
// missing_docs = built-in lint for undocumented pub items
// rustdoc::broken_intra_doc_links = broken [`Foo`] references
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod error;
mod parser;
mod tokenizer;
mod value;

pub use error::JsonError;
pub use parser::parse_json;
pub use tokenizer::{Token, tokenize};
pub use value::JsonValue;

/// Convenience alias for `std::result::Result<T, JsonError>`.
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("true").unwrap(), JsonValue::Boolean(true));
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_error_propagation() {
        let result = parse_json("@invalid@");
        assert!(result.is_err());

        match result {
            Err(JsonError::UnexpectedToken {
                expected,
                found,
                position,
            }) => {
                assert_eq!(expected, "valid JSON token");
                assert_eq!(found, "@");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}

// Only compile Python bindings when the "python" feature is active
// This lets 'cargo test' work without Python installed
#[cfg(feature = "python")]
mod python_bindings;
