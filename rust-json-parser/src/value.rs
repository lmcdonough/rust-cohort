#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl JsonValue {
    // Check if this value is the Null variant
    // matches! is a shortcut macro that returns true if the pattern matches, false otherwise
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    // Try to get the string inside. Returns None if this isn't a String variant
    // s.as_str() converts owned String to borrowed &str (more efficient, no copying)
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    // Try to get the number inside. Returns None if this isn't a Number variant
    // *n dereferences: n is a reference (&f64), *n copies the actual number out
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    // Try to get the boolean inside. Returns None if this isn't a Boolean variant
    // *b dereferences: same as *n above, copies the bool value out of the reference
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

// Copy these tests as-is:
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string());
        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }
    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());
        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);
        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));
        let value = JsonValue::Null;
        assert!(value.is_null());
    }
    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));
        assert_eq!(
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );
        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }
}
