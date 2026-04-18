use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
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

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }
}

fn escape_json_string(s: &str) -> String {
    let mut escaped = String::new();
    for ch in s.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::String(s) => write!(f, "\"{}\"", escape_json_string(s)),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{}", escape_json_string(key), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl JsonValue {
    // Kick off pretty printing with 0 depth
    pub fn pretty_print(&self, indent: usize) -> String {
        self.pretty_print_recursive(indent, 0)
    }

    // Recursive helper that tracks current depth for indentation
    fn pretty_print_recursive(&self, indent: usize, depth: usize) -> String {
        let pad = " ".repeat(indent * depth);
        let inner_pad = " ".repeat(indent * (depth + 1));

        match self {
            // Scalars just use the existing Display impl
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s),

            // Each array element on its own indented line
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| {
                        format!(
                            "{}{}",
                            inner_pad,
                            v.pretty_print_recursive(indent, depth + 1)
                        )
                    })
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), pad)
            }

            // Each key value pair on its own indented line
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}\"{}\": {}",
                            inner_pad,
                            k,
                            v.pretty_print_recursive(indent, depth + 1)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), pad)
            }
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
        let value = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert!(value.as_array().is_some());
        assert_eq!(value.as_array().unwrap().len(), 3);
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
        assert_eq!(value.get_index(5), None);
    }

    #[test]
    fn test_object_accessor() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        let value = JsonValue::Object(map);
        assert!(value.as_object().is_some());
        assert_eq!(value.as_object().unwrap().len(), 1);
        assert!(JsonValue::Null.as_object().is_none());
    }

    #[test]
    fn test_object_get() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        map.insert("age".to_string(), JsonValue::Number(30.0));
        let value = JsonValue::Object(map);
        assert_eq!(
            value.get("name"),
            Some(&JsonValue::String("Alice".to_string()))
        );
        assert_eq!(value.get("missing"), None);
        assert_eq!(JsonValue::Null.get("key"), None);
    }
}
