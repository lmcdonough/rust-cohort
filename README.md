# Rust Cohort

Exercises, assignments, and projects from a cohort-based Rust programming course.

## Projects

### rust-json-parser

A JSON tokenizer that converts raw JSON strings into a stream of tokens. Supports:

- Structural tokens: `{}`, `[]`, `,`, `:`
- Strings, numbers (integers and decimals), booleans, and null
- 15 unit tests covering edge cases (empty strings, keyword-like content in strings, invalid leading decimals)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Build & Run

```bash
cargo build
cargo run -p rust-json-parser
cargo test
```

## License

MIT
