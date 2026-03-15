# Rust Cohort

## Project Overview

Rust learning project for cohort-based coursework. Contains exercises, assignments, and projects completed during the Rust programming course.

## Project Structure

- `rust-json-parser/` — A JSON tokenizer/parser built incrementally over multiple weeks

## Tech Stack

- **Language**: Rust (latest stable)
- **Build System**: Cargo (workspace with member crates)
- **Testing**: Built-in `cargo test`

## Development Commands

```bash
# Build all projects
cargo build

# Run a specific project
cargo run -p rust-json-parser

# Run all tests
cargo test

# Run tests for a specific project
cargo test -p rust-json-parser

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run the linter
cargo clippy
```

## Code Conventions

- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types/traits)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings before committing
- Write tests for all non-trivial functions
- Use `Result` and `Option` types instead of panicking where possible
