# Rust Cohort

## Project Overview

Rust learning project for cohort-based coursework. Contains exercises, assignments, and projects completed during the Rust programming course.

## Project Structure

- `rust-json-parser/` — A JSON parser built incrementally over multiple weeks
  - `src/tokenizer.rs` — Lexer that converts JSON strings into tokens with position tracking
  - `src/parser.rs` — Parses token streams into `JsonValue` types (primitives only for now)
  - `src/error.rs` — `JsonError` enum with `UnexpectedToken`, `UnexpectedEndOfInput`, `InvalidNumber` variants
  - `src/value.rs` — `JsonValue` enum (`Null`, `Boolean`, `Number`, `String`) with accessor methods
  - `src/lib.rs` — Module declarations and public re-exports
  - `src/main.rs` — CLI entry point demonstrating tokenization

## Tech Stack

- **Language**: Rust (latest stable)
- **Build System**: Cargo
- **Testing**: Built-in `cargo test`

## Development Commands

```bash
cargo build
cargo run -p rust-json-parser
cargo test -p rust-json-parser
cargo check
cargo fmt
cargo clippy
```

## Code Conventions

- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types/traits)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings before committing
- Write tests for all non-trivial functions
- Use `Result` and `Option` types instead of panicking where possible
