# Rust Cohort

Exercises, assignments, and projects from a cohort-based Rust programming course.

## Projects

### rust-json-parser

A JSON parser built incrementally over multiple weeks. Currently supports tokenizing and parsing primitive JSON values.

**Modules:**

- **tokenizer** — Converts raw JSON strings into a stream of tokens with position tracking and error reporting
- **parser** — Parses token streams into `JsonValue` types (strings, numbers, booleans, null)
- **error** — Structured error types (`JsonError`) with position tracking for diagnostics
- **value** — `JsonValue` enum representing parsed JSON data with accessor methods

**Test coverage:** 30 tests across all modules covering tokenization, parsing, error handling, and integration.

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
