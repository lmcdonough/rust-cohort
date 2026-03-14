# Rust Cohort

## Project Overview

Rust learning project for cohort-based coursework. Contains exercises, assignments, and projects completed during the Rust programming course.

## Tech Stack

- **Language**: Rust (latest stable)
- **Build System**: Cargo
- **Testing**: Built-in `cargo test`

## Development Commands

```bash
# Build the project
cargo build

# Run the project
cargo run

# Run tests
cargo test

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
