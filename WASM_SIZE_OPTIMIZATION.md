# Typst WASM Size Optimization

Goal: Create a minimal WASM build of Typst for a math notebook application.

## Current Results

| Stage | Size |
|-------|------|
| Raw WASM | 5.7 MB |
| **Brotli** | **1.65 MB** |

### Previous Results

| Stage | Raw WASM | Brotli |
|-------|----------|--------|
| Before build optimizations | 13 MB | 3.0 MB |
| After build optimizations | 6.9 MB | 2.5 MB |
| After making HTML optional | 6.8 MB | 2.39 MB |
| After making hyphenation optional | 5.7 MB | 1.65 MB |

## Features Made Optional (Completed)

| Feature | Crate(s) Removed | Status |
|---------|------------------|--------|
| Bibliography | hayagriva | ✅ Done |
| Syntax highlighting | syntect, two-face | ✅ Done |
| Plugins/WASM interpreter | wasmi | ✅ Done |
| Raster images | image, kamadak-exif | ✅ Done |
| Regex | regex, regex-syntax | ✅ Done |
| PDF images | hayro, hayro-svg, hayro-syntax | ✅ Done |
| Data loading | csv, toml, serde_yaml | ✅ Done |
| Lorem ipsum | lipsum | ✅ Done |
| HTML export | typst-html | ✅ Done |
| Hyphenation | hypher | ✅ Done (1.1 MB raw, 0.74 MB Brotli savings) |

## Build Command

```bash
cd typst-fork
cargo build --release --target wasm32-unknown-unknown --no-default-features -p typst-wasm-test

# Optimize with wasm-opt (requires binaryen)
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals \
  target/wasm32-unknown-unknown/release/typst_wasm_test.wasm -o optimized.wasm
```

## Next Steps to Reduce Size Further

### High Impact (Likely Large Savings)

1. ~~**Remove font subsetting (subsetter, write-fonts, skrifa)**~~ ✅ Already excluded
   - These are only used by typst-pdf, which isn't in the WASM build

2. **Strip unused Unicode/ICU data (icu_*, unicode-*)**
   - Large Unicode normalization and segmentation tables
   - May be able to use smaller subsets for math-only use case
   - Estimated savings: 500KB - 1MB
   - **CJ segmentation**: ✅ Made optional (`cj-segmentation` feature) - minimal savings
   - **Main ICU data**: Still included - used for line breaking and accent handling
   - Would need simpler line-breaking fallback to remove entirely

4. **Remove or slim down font discovery (fontdb)**
   - In WASM, fonts are typically bundled, not discovered
   - Could use a minimal font loading approach
   - Estimated savings: 100-300KB

### Medium Impact

5. ~~**Make HTML export optional (typst-html)**~~ ✅ Done
   - Not needed for math notebook (SVG only)
   - Actual savings: ~100KB raw, ~40KB Brotli

6. **Remove XML/SVG parsing for import (xmlparser, roxmltree)**
   - Only needed if importing SVG images
   - Note: roxmltree is also used by usvg for SVG font glyphs
   - Estimated savings: 50-100KB (if SVG image import is disabled)

7. **Slim down math fonts**
   - Bundle only essential math glyphs
   - Use a minimal math font instead of full New Computer Modern

8. ~~**Make hyphenation optional**~~ ✅ Done
   - Not needed for math equations
   - Actual savings: 1.1 MB raw, 0.74 MB Brotli

### Build Optimizations (Completed)

| Optimization | Status |
|--------------|--------|
| `wasm-opt -Oz` instead of `-Os` | ✅ Done |
| `opt-level = "z"` in Cargo.toml | ✅ Done |
| `lto = true` (full LTO) | ✅ Done |
| `codegen-units = 1` | ✅ Done |
| `strip = true` for WASM package | ✅ Done |

## Files Modified

### Cargo.toml (workspace root)
- Changed `lto = "thin"` to `lto = true` for full LTO
- Added `opt-level = "z"` for size optimization
- Added package-specific `strip = true` for typst-wasm-test

### typst-library/Cargo.toml
- Made `hayagriva`, `syntect`, `two-face`, `wasmi`, `image`, `kamadak-exif`, `regex`, `regex-syntax` optional
- Added feature flags: `bibliography`, `syntax-highlighting`, `plugins`, `raster-images`, `regex`

### typst-realize/Cargo.toml
- Made `regex` optional
- Added feature flag propagation

### typst-layout/Cargo.toml
- Added `regex` feature flag
- Added `hyphenation` feature flag (makes `hypher` optional)

### typst-svg/Cargo.toml
- Added `raster-images` feature flag

### typst/Cargo.toml
- Added feature propagation for all optional features

### Source Files with Conditional Compilation
- `typst-library/src/foundations/str.rs` - Regex type and StrPattern
- `typst-library/src/foundations/selector.rs` - Selector::Regex
- `typst-library/src/foundations/func.rs` - Plugin support
- `typst-library/src/visualize/image/mod.rs` - Raster image types
- `typst-layout/src/inline/shaping.rs` - Font covers (regex)
- `typst-layout/src/inline/linebreak.rs` - Hyphenation support
- `typst-realize/src/lib.rs` - Regex show rules
- `typst-svg/src/text.rs` - Bitmap glyph rendering
- `typst-svg/src/image.rs` - Raster image conversion
- `typst/src/lib.rs` - HTML export support
