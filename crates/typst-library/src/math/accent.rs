use std::collections::HashMap;
use std::sync::LazyLock;

use bumpalo::Bump;
use comemo::Tracked;
#[cfg(feature = "icu-properties")]
use icu_properties::CanonicalCombiningClass;
#[cfg(feature = "icu-properties")]
use icu_properties::maps::CodePointMapData;
#[cfg(feature = "icu-properties")]
use icu_provider::AsDeserializingBufferProvider;
#[cfg(feature = "icu-properties")]
use icu_provider_blob::BlobDataProvider;

use crate::engine::Engine;
use crate::foundations::{
    Args, CastInfo, Content, Context, Func, IntoValue, NativeElement, NativeFuncData,
    NativeFuncPtr, ParamInfo, Reflect, Scope, Str, SymbolElem, Type, cast, elem,
};
use crate::layout::{Length, Rel};
use crate::math::Mathy;

/// Attaches an accent to a base.
///
/// # Example
/// ```example
/// $grave(a) = accent(a, `)$ \
/// $arrow(a) = accent(a, arrow)$ \
/// $tilde(a) = accent(a, \u{0303})$
/// ```
#[elem(Mathy)]
pub struct AccentElem {
    /// The base to which the accent is applied. May consist of multiple
    /// letters.
    ///
    /// ```example
    /// $arrow(A B C)$
    /// ```
    #[required]
    pub base: Content,

    /// The accent to apply to the base.
    ///
    /// Supported accents include:
    ///
    /// | Accent        | Name            | Codepoint |
    /// | ------------- | --------------- | --------- |
    /// | Grave         | `grave`         | <code>&DiacriticalGrave;</code> |
    /// | Acute         | `acute`         | `´`       |
    /// | Circumflex    | `hat`           | `^`       |
    /// | Tilde         | `tilde`         | `~`       |
    /// | Macron        | `macron`        | `¯`       |
    /// | Dash          | `dash`          | `‾`       |
    /// | Breve         | `breve`         | `˘`       |
    /// | Dot           | `dot`           | `.`       |
    /// | Double dot, Diaeresis | `dot.double`, `diaer` | `¨` |
    /// | Triple dot    | `dot.triple`    | <code>&tdot;</code> |
    /// | Quadruple dot | `dot.quad`      | <code>&DotDot;</code> |
    /// | Circle        | `circle`        | `∘`       |
    /// | Double acute  | `acute.double`  | `˝`       |
    /// | Caron         | `caron`         | `ˇ`       |
    /// | Right arrow   | `arrow`, `->`   | `→`       |
    /// | Left arrow    | `arrow.l`, `<-` | `←`       |
    /// | Left/Right arrow | `arrow.l.r`  | `↔`       |
    /// | Right harpoon | `harpoon`       | `⇀`       |
    /// | Left harpoon  | `harpoon.lt`    | `↼`       |
    #[required]
    pub accent: Accent,

    /// The size of the accent, relative to the width of the base.
    ///
    /// ```example
    /// $dash(A, size: #150%)$
    /// ```
    #[default(Rel::one())]
    pub size: Rel<Length>,

    /// Whether to remove the dot on top of lowercase i and j when adding a top
    /// accent.
    ///
    /// This enables the `dtls` OpenType feature.
    ///
    /// ```example
    /// $hat(dotless: #false, i)$
    /// ```
    #[default(true)]
    pub dotless: bool,
}

/// An accent character.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Accent(pub char);

impl Accent {
    /// Tries to select the appropriate combining accent for a string, falling
    /// back to the string's lone character if there is no corresponding one.
    ///
    /// Returns `None` if there isn't one and the string has more than one
    /// character.
    pub fn normalize(s: &str) -> Option<Self> {
        Self::combining(s).or_else(|| s.parse::<char>().ok().map(Self))
    }

    /// Tries to select a well-known combining accent that matches for the
    /// value.
    pub fn combining(value: &str) -> Option<Self> {
        let c = value.parse::<char>().ok();
        ACCENTS
            .iter()
            .copied()
            .find(|&(accent, names)| Some(accent) == c || names.contains(&value))
            .map(|(accent, _)| Self(accent))
    }

    /// Whether this accent is a bottom accent or not (ICU-based).
    #[cfg(feature = "icu-properties")]
    pub fn is_bottom(&self) -> bool {
        if matches!(self.0, '⏟' | '⎵' | '⏝' | '⏡') {
            return true;
        }

        static COMBINING_CLASS_DATA: LazyLock<CodePointMapData<CanonicalCombiningClass>> =
            LazyLock::new(|| {
                icu_properties::maps::load_canonical_combining_class(
                    &BlobDataProvider::try_new_from_static_blob(typst_assets::icu::ICU)
                        .unwrap()
                        .as_deserializing(),
                )
                .unwrap()
            });

        matches!(
            COMBINING_CLASS_DATA.as_borrowed().get(self.0),
            CanonicalCombiningClass::Below
        )
    }

    /// Whether this accent is a bottom accent or not (hardcoded fallback).
    ///
    /// This uses a hardcoded list of common bottom combining characters
    /// instead of the full ICU data.
    #[cfg(not(feature = "icu-properties"))]
    pub fn is_bottom(&self) -> bool {
        // Special bottom accent characters
        if matches!(self.0, '⏟' | '⎵' | '⏝' | '⏡') {
            return true;
        }

        // Combining characters with Canonical Combining Class = Below (220)
        // This is a subset of the most common ones used in math/typography
        matches!(self.0,
            // Common below combining marks
            '\u{0316}' |  // COMBINING GRAVE ACCENT BELOW
            '\u{0317}' |  // COMBINING ACUTE ACCENT BELOW
            '\u{0318}' |  // COMBINING LEFT TACK BELOW
            '\u{0319}' |  // COMBINING RIGHT TACK BELOW
            '\u{031C}' |  // COMBINING LEFT HALF RING BELOW
            '\u{031D}' |  // COMBINING UP TACK BELOW
            '\u{031E}' |  // COMBINING DOWN TACK BELOW
            '\u{031F}' |  // COMBINING PLUS SIGN BELOW
            '\u{0320}' |  // COMBINING MINUS SIGN BELOW
            '\u{0321}' |  // COMBINING PALATALIZED HOOK BELOW
            '\u{0322}' |  // COMBINING RETROFLEX HOOK BELOW
            '\u{0323}' |  // COMBINING DOT BELOW
            '\u{0324}' |  // COMBINING DIAERESIS BELOW
            '\u{0325}' |  // COMBINING RING BELOW
            '\u{0326}' |  // COMBINING COMMA BELOW
            '\u{0327}' |  // COMBINING CEDILLA
            '\u{0328}' |  // COMBINING OGONEK
            '\u{0329}' |  // COMBINING VERTICAL LINE BELOW
            '\u{032A}' |  // COMBINING BRIDGE BELOW
            '\u{032B}' |  // COMBINING INVERTED DOUBLE ARCH BELOW
            '\u{032C}' |  // COMBINING CARON BELOW
            '\u{032D}' |  // COMBINING CIRCUMFLEX ACCENT BELOW
            '\u{032E}' |  // COMBINING BREVE BELOW
            '\u{032F}' |  // COMBINING INVERTED BREVE BELOW
            '\u{0330}' |  // COMBINING TILDE BELOW
            '\u{0331}' |  // COMBINING MACRON BELOW
            '\u{0332}' |  // COMBINING LOW LINE
            '\u{0333}' |  // COMBINING DOUBLE LOW LINE
            '\u{0339}' |  // COMBINING RIGHT HALF RING BELOW
            '\u{033A}' |  // COMBINING INVERTED BRIDGE BELOW
            '\u{033B}' |  // COMBINING SQUARE BELOW
            '\u{033C}' |  // COMBINING SEAGULL BELOW
            '\u{0345}' |  // COMBINING GREEK YPOGEGRAMMENI (iota subscript)
            '\u{0347}' |  // COMBINING EQUALS SIGN BELOW
            '\u{0348}' |  // COMBINING DOUBLE VERTICAL LINE BELOW
            '\u{0349}' |  // COMBINING LEFT ANGLE BELOW
            '\u{034D}' |  // COMBINING LEFT RIGHT ARROW BELOW
            '\u{034E}' |  // COMBINING UPWARDS ARROW BELOW
            '\u{0353}' |  // COMBINING X BELOW
            '\u{0354}' |  // COMBINING LEFT ARROWHEAD BELOW
            '\u{0355}' |  // COMBINING RIGHT ARROWHEAD BELOW
            '\u{0356}' |  // COMBINING RIGHT ARROWHEAD AND UP ARROWHEAD BELOW
            '\u{0359}' |  // COMBINING ASTERISK BELOW
            '\u{035A}' |  // COMBINING DOUBLE RING BELOW
            '\u{035C}' |  // COMBINING DOUBLE BREVE BELOW
            '\u{035F}' |  // COMBINING DOUBLE MACRON BELOW
            '\u{0362}'    // COMBINING DOUBLE RIGHTWARDS ARROW BELOW
        )
    }
}

/// Gets the accent function corresponding to a symbol value, if any.
pub fn get_accent_func(value: &str) -> Option<Func> {
    Accent::combining(value).map(|accent| (&FUNCS[&accent]).into())
}

// Keep it synced with the documenting table above and the
// `math-accent-sym-call` test.`
/// A list of accents, each with a list of alternative names.
const ACCENTS: &[(char, &[&str])] = &[
    // Note: Symbols that can have a text presentation must explicitly have that
    // alternative listed here.
    ('\u{0300}', &["`"]),
    ('\u{0301}', &["´"]),
    ('\u{0302}', &["^", "ˆ"]),
    ('\u{0303}', &["~", "∼", "˜"]),
    ('\u{0304}', &["¯"]),
    ('\u{0305}', &["-", "–", "‾", "−"]),
    ('\u{0306}', &["˘"]),
    ('\u{0307}', &[".", "˙", "⋅"]),
    ('\u{0308}', &["¨"]),
    ('\u{20db}', &[]),
    ('\u{20dc}', &[]),
    ('\u{030a}', &["∘", "○"]),
    ('\u{030b}', &["˝"]),
    ('\u{030c}', &["ˇ"]),
    ('\u{20d6}', &["←"]),
    ('\u{20d7}', &["→", "⟶"]),
    ('\u{20e1}', &["↔", "↔\u{fe0e}", "⟷"]),
    ('\u{20d0}', &["↼"]),
    ('\u{20d1}', &["⇀"]),
];

/// Lazily created accent functions.
static FUNCS: LazyLock<HashMap<Accent, NativeFuncData>> = LazyLock::new(|| {
    let bump = Box::leak(Box::new(Bump::new()));
    ACCENTS
        .iter()
        .copied()
        .map(|(accent, _)| (Accent(accent), create_accent_func_data(accent, bump)))
        .collect()
});

/// Creates metadata for an accent wrapper function.
fn create_accent_func_data(accent: char, bump: &'static Bump) -> NativeFuncData {
    let title = bumpalo::format!(in bump, "Accent ({})", accent).into_bump_str();
    let docs = bumpalo::format!(in bump, "Adds the accent {} on an expression.", accent)
        .into_bump_str();
    NativeFuncData {
        function: NativeFuncPtr(bump.alloc(
            move |_: &mut Engine, _: Tracked<Context>, args: &mut Args| {
                let base = args.expect("base")?;
                let size = args.named("size")?;
                let dotless = args.named("dotless")?;
                let mut elem = AccentElem::new(base, Accent(accent));
                if let Some(size) = size {
                    elem = elem.with_size(size);
                }
                if let Some(dotless) = dotless {
                    elem = elem.with_dotless(dotless);
                }
                Ok(elem.pack().into_value())
            },
        )),
        name: "(..) => ..",
        title,
        docs,
        keywords: &[],
        contextual: false,
        scope: LazyLock::new(&|| Scope::new()),
        params: LazyLock::new(&|| create_accent_param_info()),
        returns: LazyLock::new(&|| CastInfo::Type(Type::of::<Content>())),
    }
}

/// Creates parameter signature metadata for an accent function.
fn create_accent_param_info() -> Vec<ParamInfo> {
    vec![
        ParamInfo {
            name: "base",
            docs: "The base to which the accent is applied.",
            input: Content::input(),
            default: None,
            positional: true,
            named: false,
            variadic: false,
            required: true,
            settable: false,
        },
        ParamInfo {
            name: "size",
            docs: "The size of the accent, relative to the width of the base.",
            input: Rel::<Length>::input(),
            default: None,
            positional: false,
            named: true,
            variadic: false,
            required: false,
            settable: false,
        },
        ParamInfo {
            name: "dotless",
            docs: "Whether to remove the dot on top of lowercase i and j when adding a top accent.",
            input: bool::input(),
            default: None,
            positional: false,
            named: true,
            variadic: false,
            required: false,
            settable: false,
        },
    ]
}

cast! {
    Accent,
    self => self.0.into_value(),
    // The string cast handles
    // - strings: `accent(a, "↔")`
    // - symbol values: `accent(a, <->)`
    // - shorthands: `accent(a, arrow.l.r)`
    v: Str => Self::normalize(&v).ok_or("expected exactly one character")?,
    // The content cast is for accent uses like `accent(a, ↔)`
    v: Content => v.to_packed::<SymbolElem>()
        .and_then(|elem| Accent::normalize(&elem.text))
        .ok_or("expected a single-codepoint symbol")?,
}
