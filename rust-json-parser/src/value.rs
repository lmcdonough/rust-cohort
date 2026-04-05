use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>), // A Json Array: ordered list of any JSON values, Vec<JsonValue>: a growable heap allocated list that owns its elements
    Object(HashMap<String, JsonValue>), // unordered dict, String keys bc HashMap must Own its keys
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    // Returns a reference to the Vec inside Array, or None for any other variant.
    // &self = borrow self (dont consume it)
    // Option<&Vec<JsonValue>> = either a pointer to the Vec, or nothing
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr), // arr is already &Vec here
            _ => None,
        }
    }

    // Returns a reference to the HashMap inside Object, or None for any other variant
    // Exact same pattern as as_array() - just a different variant
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    // Gets an element at index inside an Array variant
    // index: usize - array indices are always non-negative in Rust
    // Delegates to Vec's own .get() which returns Option<&T> (None if out of bounds)
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index), // Vec::get returns None if out of bounds
            _ => None, // if called on Object, Number, etc. - just none
        }
    }

    // Looks up a key inside an Object variant
    // key: &str - HashMap<String,_> accepts &str for lookups via Rust's Borrow trait
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }
}

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
    #[test]
    fn test_array_accessor() {
        // Construct an Array directly (parse_array comes in Step 3)
        let value = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert!(value.as_array().is_some());
        assert_eq!(value.as_array().unwrap().len(), 3);
        // Wrong variant → None
        assert!(JsonValue::Null.as_array().is_none());
    }

    #[test]
    fn test_array_get_index() {
        let value = JsonValue::Array(vec![
            JsonValue::Number(10.0),
            JsonValue::Number(20.0),
            JsonValue::Number(30.0),
        ]);
        assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
        assert_eq!(value.get_index(5), None); // out of bounds → None
    }

    #[test]
    fn test_object_accessor() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        let value = JsonValue::Object(map);
        assert!(value.as_object().is_some());
        assert_eq!(value.as_object().unwrap().len(), 1);
        // Wrong variant → None
        assert!(JsonValue::Null.as_object().is_none());
    }

    #[test]
    fn test_object_get() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        map.insert("age".to_string(), JsonValue::Number(30.0));
        let value = JsonValue::Object(map);
        // &str lookup — no .to_string() needed
        assert_eq!(
            value.get("name"),
            Some(&JsonValue::String("Alice".to_string()))
        );
        assert_eq!(value.get("missing"), None);
        // Wrong variant → None
        assert_eq!(JsonValue::Null.get("key"), None);
    }
}
