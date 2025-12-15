# ICU Data Optimization Plan

## Goal
Reduce WASM size by making ICU (International Components for Unicode) data optional or using smaller alternatives.

## Current State
- Raw WASM: 4.5 MB
- Brotli: 1.49 MB
- Estimated ICU contribution: 500KB - 1MB

## ICU Data Source
ICU data is embedded via the `typst-assets` crate:
- Repository: https://github.com/typst/typst-assets
- Current rev: `6eed5f2`
- Data blobs:
  - `typst_assets::icu::ICU` - Main ICU data blob
  - `typst_assets::icu::ICU_CJ_SEGMENT` - CJK segmentation (already optional via `cj-segmentation` feature)

## Where ICU Is Used

### 1. Line Breaking (`typst-layout/src/inline/linebreak.rs`)
**Crates:** `icu_segmenter`, `icu_properties`, `icu_provider`, `icu_provider_adapters`, `icu_provider_blob`

**Usage:**
```rust
// Line segmenter for determining where lines can break
static SEGMENTER: LazyLock<LineSegmenter> =
    LazyLock::new(|| LineSegmenter::try_new_lstm_with_buffer_provider(&blob()).unwrap());

// Line break properties for each Unicode code point
static LINEBREAK_DATA: LazyLock<CodePointMapData<LineBreak>> = LazyLock::new(|| {
    icu_properties::maps::load_line_break(&blob().as_deserializing()).unwrap()
});
```

**Purpose:** Determines valid line break opportunities according to Unicode Line Breaking Algorithm (UAX #14).

### 2. Math Accents (`typst-library/src/math/accent.rs`)
**Crates:** `icu_properties`, `icu_provider`, `icu_provider_blob`

**Usage:**
```rust
// Canonical combining class for accent positioning
icu_properties::maps::load_canonical_combining_class(
    &BlobDataProvider::try_new_from_static_blob(typst_assets::icu::ICU)
        .unwrap()
        .as_deserializing(),
)
```

**Purpose:** Determines the combining class of characters to correctly position accents in math mode.

### 3. Default Ignorable Code Points (`typst-library/src/text/mod.rs`)
**Crates:** `icu_properties`, `icu_provider`, `icu_provider_blob`

**Usage:**
```rust
// Characters that should be ignored in rendering
icu_properties::sets::load_default_ignorable_code_point(
    &BlobDataProvider::try_new_from_static_blob(typst_assets::icu::ICU)
        .unwrap()
        .as_deserializing(),
)
```

**Purpose:** Identifies characters like zero-width spaces that shouldn't produce visible output.

## Optimization Strategies

### Strategy A: Simple Line Breaking Fallback (Recommended for Math Notebook)
**Estimated savings:** 300-500KB

For math-only content, we don't need sophisticated line breaking. Implement a simple fallback:

1. Create `simple-linebreak` feature flag
2. When enabled, use basic rules:
   - Break after spaces
   - Break after hyphens
   - Break at explicit `\n`
   - No CJK support needed

**Files to modify:**
- `crates/typst-layout/Cargo.toml` - Add feature flag, make icu_segmenter optional
- `crates/typst-layout/src/inline/linebreak.rs` - Add cfg-gated simple implementation

**Fallback implementation sketch:**
```rust
#[cfg(not(feature = "icu-linebreak"))]
fn simple_line_opportunities(text: &str) -> impl Iterator<Item = usize> {
    text.char_indices()
        .filter(|(_, c)| c.is_whitespace() || *c == '-')
        .map(|(i, c)| i + c.len_utf8())
}
```

### Strategy B: Hardcoded Combining Classes (For Math Accents)
**Estimated savings:** 50-100KB

Math accents only use a small set of combining characters. Hardcode them:

1. Create lookup table for common math accent combining classes
2. Fall back to a default class for unknown characters

**Files to modify:**
- `crates/typst-library/src/math/accent.rs` - Add hardcoded table

### Strategy C: Minimal ICU Data Blob
**Estimated savings:** 200-400KB

Fork `typst-assets` and create a minimal ICU blob with only:
- Basic line break properties (ASCII + common punctuation)
- Combining classes for math accents
- Default ignorable set

**Complexity:** High - requires understanding ICU data format and `icu_datagen` tool.

### Strategy D: Remove Default Ignorable Check
**Estimated savings:** 50KB

For math notebook, this check may not be necessary. Simply skip the check.

**Files to modify:**
- `crates/typst-library/src/text/mod.rs` - Gate behind feature flag

## Recommended Implementation Order

1. **Strategy A** - Simple line breaking (biggest win, most useful for math)
2. **Strategy D** - Remove default ignorable check (easy)
3. **Strategy B** - Hardcoded combining classes (if more savings needed)
4. **Strategy C** - Minimal ICU blob (last resort, complex)

## Dependencies to Make Optional

| Crate | Used By | Can Remove? |
|-------|---------|-------------|
| `icu_segmenter` | linebreak.rs | Yes, with simple fallback |
| `icu_segmenter_data` | linebreak.rs | Yes, with simple fallback |
| `icu_properties` | linebreak.rs, accent.rs, text/mod.rs | Partially, need hardcoded tables |
| `icu_properties_data` | (via icu_properties) | Partially |
| `icu_provider` | All ICU usage | Only if all ICU removed |
| `icu_provider_blob` | All ICU usage | Only if all ICU removed |
| `icu_provider_adapters` | linebreak.rs | Yes, with simple fallback |

## Testing Considerations

After implementing, verify:
1. Basic text still renders correctly
2. Math equations with accents work: `$\hat{x}$`, `$\vec{v}$`
3. Line breaks happen at reasonable places
4. No panics on edge cases (empty text, unicode)

## Files Summary

| File | ICU Usage | Optimization |
|------|-----------|--------------|
| `crates/typst-layout/src/inline/linebreak.rs` | LineSegmenter, LineBreak properties | Simple fallback |
| `crates/typst-library/src/math/accent.rs` | CanonicalCombiningClass | Hardcoded table |
| `crates/typst-library/src/text/mod.rs` | DefaultIgnorableCodePoint | Remove check |
| `crates/typst-layout/Cargo.toml` | Dependencies | Feature flags |
| `crates/typst-library/Cargo.toml` | Dependencies | Feature flags |
