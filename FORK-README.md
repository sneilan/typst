# Typst Fork for Minimal WASM Build

This is a fork of [typst/typst](https://github.com/typst/typst) with optional bibliography and syntax highlighting features to reduce WASM size.

## Status: Working! 29% size reduction achieved

The fork successfully compiles with optional features disabled. Feature flags have been propagated through all dependent crates in the workspace.

### Final WASM Size Results

| Build Configuration | WASM Size | Brotli Size |
|---------------------|-----------|-------------|
| **Full** (bibliography + syntax-highlighting) | 28 MB | 6.2 MB |
| **Minimal** (no optional features) | 21 MB | 4.7 MB |
| **Minimal + wasm-strip** | 19 MB | **4.4 MB** |
| **Savings** | 9 MB (32%) | **1.8 MB (29%)** |

### Why Not Smaller?

The remaining ~4.4MB brotli is the core typst engine which includes:
- **rustybuzz** - Text shaping engine (essential for text)
- **skrifa** - Font hinting and outline rendering
- **wasmi** - WASM interpreter for plugins (~500KB, could be made optional)
- **image/hayro** - Image decoding and PDF rendering (~400KB, could be made optional)
- **regex_automata** - Pattern matching
- **ICU data** - Unicode handling

For a pure math-only use case, more could potentially be stripped (plugins, images), but that would require deeper changes to typst-library.

## What Was Done

### 1. Made Dependencies Optional in `typst-library`

```toml
[features]
default = ["bibliography", "syntax-highlighting"]
bibliography = ["dep:hayagriva"]
syntax-highlighting = ["dep:syntect", "dep:two-face"]
```

### 2. Added cfg Guards Throughout

- **`model/reference.rs`**: Changed `citation` field type, wrapped BibliographyElem usage
- **`model/quote.rs`**: Added fallback for Attribution::Label
- **`text/raw.rs`**: Added stub types and non-highlighting version of `highlight()`
- **`model/mod.rs`**: Wrapped bibliography/cite module imports and exports

### 3. Propagated Features Through Workspace

Updated these crates to use `default-features = false` and propagate the bibliography feature:

- `crates/typst/Cargo.toml`
- `crates/typst-layout/Cargo.toml`
- `crates/typst-realize/Cargo.toml`
- `crates/typst-html/Cargo.toml`
- `Cargo.toml` (workspace)

### 4. Added cfg Guards to Dependent Crates

- **`typst-layout/src/rules.rs`**: Wrapped CITE_GROUP_RULE, BIBLIOGRAPHY_RULE, etc.
- **`typst-realize/src/lib.rs`**: Wrapped CITES static, finish_cites function, and rule arrays
- **`typst-html/src/rules.rs`**: Wrapped bibliography imports and rules

## Build Commands

### Minimal Build (no bibliography, no syntax highlighting)
```bash
cargo build --release --target wasm32-unknown-unknown -p typst --no-default-features
```

### Full Build
```bash
cargo build --release --target wasm32-unknown-unknown -p typst
```

### Test WASM Sizes
```bash
# Build minimal
cargo build --release --target wasm32-unknown-unknown -p typst-wasm-test --no-default-features

# Strip debug info
wasm-tools strip --all target/wasm32-unknown-unknown/release/typst_wasm_test.wasm \
  -o target/wasm32-unknown-unknown/release/typst_wasm_test_stripped.wasm

# Compress with brotli
brotli --best target/wasm32-unknown-unknown/release/typst_wasm_test_stripped.wasm

# Check size
ls -lh target/wasm32-unknown-unknown/release/typst_wasm_test_stripped.wasm.br
```

## Potential Further Optimizations

These would require deeper changes but could save more:

1. **Make plugin support (wasmi) optional** - ~500KB savings
2. **Make image support optional** - ~400KB savings (image, hayro, zune-jpeg)
3. **Custom build of ICU data** - Reduce Unicode data tables

## Comparison with Alternatives

| Option | Brotli Size | Syntax |
|--------|-------------|--------|
| KaTeX | ~300 KB | LaTeX (verbose) |
| MathJax | ~1.5 MB | LaTeX |
| **This Typst Fork** | **4.4 MB** | Typst (clean) |
| Full Typst | 6.2 MB | Typst |

Typst's syntax is much cleaner than LaTeX, but the engine is larger because it's a full typesetting system, not just a math renderer.

---

*Fork created: December 2024*
*Based on: typst/typst main branch*
