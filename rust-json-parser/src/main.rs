use rust_json_parser::{parse_json, tokenize};

fn main() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    println!("Input JSON: {}", json);

    let tokens = tokenize(json);
    println!("Tokens:\n{:?}", tokens);

    let values = vec![r#""hello world""#, "42", "true", "null", "@invalid"];

    for input in values {
        match parse_json(input) {
            Ok(value) => println!("Parsed '{}' => {:?}", input, value),
            Err(e) => println!("Error parsing '{}': {}", input, e),
        }
    }
}
