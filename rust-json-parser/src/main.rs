use rust_json_parser::tokenizer::tokenize;

fn main() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    println!("Input JSON: {}", json);

    let tokens = tokenize(json);
    println!("Tokens:\n{:?}", tokens);
}
