# Rust Cohort

## Project Overview

Rust learning project for cohort-based coursework. Contains exercises, assignments, and projects completed during the Rust programming course.

## Project Structure

- `rust-json-parser/` — A JSON parser + Python bindings built incrementally over multiple weeks
  - `src/tokenizer.rs` — Lexer that converts JSON strings into tokens with position tracking
  - `src/parser.rs` — Parses token streams into `JsonValue` (primitives, arrays, objects, nested)
  - `src/error.rs` — `JsonError` enum: `UnexpectedToken`, `UnexpectedEndOfInput`, `InvalidNumber`, `InvalidEscape`, `InvalidUnicode`
  - `src/value.rs` — `JsonValue` enum (`Null`, `Boolean`, `Number`, `String`, `Array`, `Object`) with accessors, `Display`, and `pretty_print`
  - `src/python_bindings.rs` — PyO3 bindings gated behind the `python` cargo feature
  - `src/lib.rs` — Module declarations and public re-exports
  - `src/main.rs` — CLI entry point demonstrating tokenization
  - `python/rust_json_parser/` — Python package (editable install via maturin)
    - `__init__.py` — re-exports `parse_json`, `parse_json_file`, `dumps`, `benchmark_performance` from the compiled extension
    - `__main__.py` — `python -m rust_json_parser` CLI (stdin / file / literal arg, plus `--benchmark` mode vs `json` (C) and `simplejson`)
  - `tests/test_python_integration.py` — pytest suite for the Python bindings (45 tests)
  - `Cargo.toml` — crate manifest; `python` is an opt-in feature, not default
  - `pyproject.toml` — maturin build config; activates the `python` cargo feature for wheel builds; declares runtime deps (`typer`, `simplejson`)
  - `benchmarks_baseline.md` / `benchmarks_optimized.md` — pre- and post-optimization benchmark numbers with deltas and analysis (week 6)

## Tech Stack

- **Language**: Rust (latest stable), Python 3.12+
- **Build System**: Cargo (Rust), maturin (Python extension)
- **Python FFI**: PyO3 0.27
- **Testing**: `cargo test` (Rust), `pytest` (Python integration)

## Cargo Features

- `default = []` — no features on by default, so `cargo test` builds without linking to libpython
- `python` — enables `pyo3` with `extension-module`; activated by maturin via `pyproject.toml`

## Development Commands

All commands run from inside `rust-json-parser/` (the crate is not a workspace member).

```bash
# Rust
cargo build
cargo test                       # runs 65 unit/integration tests, no Python required
cargo check --features python    # verify bindings still compile
cargo fmt -- --check
cargo clippy -- -D warnings

# Python bindings (requires venv with maturin + pytest installed)
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest typer simplejson
maturin develop                  # debug build; use --release for benchmarking
pytest tests/test_python_integration.py
python -m rust_json_parser '{"hi": 1}'
python -m rust_json_parser --benchmark   # compare vs json (C) and simplejson
```

## CI

GitHub Actions workflow at `.github/workflows/ci.yml` runs on pushes to `main`/`week*` and PRs to `main`. It exercises the full local validation surface: build, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings`, plus the Python integration tests (build the extension with `maturin develop`, then `pytest`).

## Performance Notes (week 6)

- Hot path allocates `String`/`Vec` with `with_capacity` to skip early reallocs (see `src/tokenizer.rs` and `parse_array` in `src/parser.rs`).
- `parse_value` and `parse_object_key` use `std::mem::take` to move `String` out of `Token::String` slots instead of cloning — the parser advances past each slot exactly once, so leaving `String::default()` behind is safe.
- When adding new parser code, prefer `match` over `map_err` closures if the error needs to consume an owned buffer — closures force a clone because the buffer may still be needed on the success path.
- Benchmark with a release build (`maturin develop --release`) before comparing numbers; debug builds are 10–100x slower.

## Code Conventions

- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types/traits)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings before committing
- Write tests for all non-trivial functions
- Use `Result` and `Option` types instead of panicking where possible
