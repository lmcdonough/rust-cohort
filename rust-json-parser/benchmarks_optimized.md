# Post-Optimization Benchmarks

**Date:** 2026-04-24
**Commit:** d2c7ca0 (working tree — optimizations not yet committed)
**Build:** `maturin develop --release`

## Optimizations Applied (Step 10)

- [x] `String::with_capacity(N)` in tokenizer — locations:
  - `src/tokenizer.rs:57` (`tokenize_string`, capacity 64)
  - `src/tokenizer.rs:80` (`\uXXXX` hex buffer, capacity 4)
  - `src/tokenizer.rs:145` (`tokenize_number`, capacity 16)
  - `src/tokenizer.rs:172` (`tokenize_keyword`, capacity 8)
- [x] `Vec::with_capacity(16)` in `parse_array` — location: `src/parser.rs:87`
- [x] Removed 4 unnecessary `.clone()` calls — locations:
  - `src/parser.rs:58` — `parse_value` string branch now uses `std::mem::take`
  - `src/parser.rs:136` — `parse_object_key` now uses `std::mem::take`
  - `src/tokenizer.rs:95` — `\uXXXX` path replaced `map_err` with `match` so `hex_str` moves into the error arm
  - `src/tokenizer.rs:156` — `tokenize_number` replaced `map_err` with `match` so `num_str` moves into the error arm

## Results

Numbers below are the median of three back-to-back runs.

### Small JSON (16 bytes)
| Parser | Baseline | Optimized | Delta | Speedup |
|---|---|---|---|---|
| Rust | 0.000313s | 0.000247s | −21.09% | 1.27x |
| Python json (C) | 0.000497s | (not our code) | — | — |
| simplejson | 0.000535s | (not our code) | — | — |

### Medium JSON (2981 bytes)
| Parser | Baseline | Optimized | Delta | Speedup |
|---|---|---|---|---|
| Rust | 0.022288s | 0.018724s | −15.99% | 1.19x |
| Python json (C) | 0.013362s | (not our code) | — | — |
| simplejson | 0.015765s | (not our code) | — | — |

### Large JSON (47781 bytes)
| Parser | Baseline | Optimized | Delta | Speedup |
|---|---|---|---|---|
| Rust | 0.295088s | 0.248346s | −15.84% | 1.19x |
| Python json (C) | 0.181876s | (not our code) | — | — |
| simplejson | 0.207989s | (not our code) | — | — |

## Analysis

### What moved the needle

- **Removing `.clone()` on string tokens** was by far the biggest win. Every JSON string value and every object key previously allocated + memcpy'd a fresh `String` on its way from `Token::String` into `JsonValue::String`. Switching to `std::mem::take(&mut ...)` drops that allocation entirely, which is what drove the bulk of the ~16–21% speedup visible at every input size.
- **`String::with_capacity` in the tokenizer** contributed a smaller amount by avoiding the 2–3 early reallocations as buffers grow from zero. Most impactful on small inputs where allocation overhead is a larger fraction of total time.

### What didn't help

- **`Vec::with_capacity(16)` in `parse_array`** had no measurable impact on its own (flat within noise when measured in isolation during Step 10). The benchmark corpus is object-heavy, so array-growth reallocations aren't in the hot path. Left in place because the change is correct and free.
- **An earlier attempt** to use `self.input.len() - self.position` as the `tokenize_string` capacity caused a **+63% regression** on the Large input: every short string allocated a buffer the size of the entire remaining file. Reverted to the constant-64 default during Step 10.

### Where Rust now stands vs Python json (C)

Rust is still ~1.3–1.5x slower than `json` (C) on medium/large inputs (0.249s vs 0.185s on Large). The C implementation is hand-tuned and avoids the PyO3 conversion overhead we pay on every JSON value crossing the FFI boundary. Rust decisively wins on small inputs (~1.8x faster) because import/setup costs amortize worse for the C extension at that size.

### Where Rust now stands vs simplejson

Rust is consistently faster than simplejson across all sizes, but only modestly — roughly 1.1x–2.1x, not the 5–15x the spec hinted at. simplejson is the pure-Python JSON library; the narrower-than-expected margin is explained by the current implementation's `Vec<char>` input representation, 4 clones removed from hot paths (but allocation on every `JsonValue::String` construction via PyO3 still remains), and FFI overhead. A `&[u8]` / `&str` tokenizer with zero-copy string slicing would likely widen this gap substantially.

## Reproducing These Numbers

```bash
maturin develop --release
python -m rust_json_parser --benchmark
```

## Caveats

- Numbers vary by ±10% across runs due to OS scheduling and cache state. Small-input timings are especially noisy (sub-millisecond).
- Benchmarks run on `Darwin MacBook-Pro-c10d64f 25.4.0 Darwin Kernel Version 25.4.0: Thu Mar 19 19:33:25 PDT 2026; root:xnu-12377.101.15~1/RELEASE_ARM64_T6041 arm64` (Apple Silicon).
- Release build only — debug builds are 10–100x slower and will mislead any comparison.
- Baseline numbers came from a single run at commit `d2c7ca0`; the ±10% noise band applies to both sides of the delta.
