//! The renderer contract — the one rendering-related thing the protocol defines.
//!
//! Stated entirely over the spec's generated types (it is handed a [`Font`], never a
//! string — so it can only be given something the spec defines). Implementations live in
//! their own crates (cascade-css, cascade-typst), depend on this one, bring their own
//! templates, and supply target BEHAVIOR — like how a font resolves to a family with
//! fallbacks. The protocol renders nothing itself.
use crate::{
    Category, Font, ScalePreset, Theme, FONT_BODY, FONT_CODE, FONT_HEADING, SCALE_DEFAULT, THEME_DEFAULT,
};

/// One generated output file: a target-relative path and its contents.
pub struct Output {
    pub path: String,
    pub body: String,
}

/// A font file format, for delivery (`@font-face` MIME + `format()` hint).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FontFormat {
    Ttf,
    Otf,
    Woff2,
}

impl FontFormat {
    /// From a file extension (case-insensitive); `None` if not a known font format.
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "ttf" => Some(Self::Ttf),
            "otf" => Some(Self::Otf),
            "woff2" => Some(Self::Woff2),
            _ => None,
        }
    }
    /// MIME type for a `data:` URI.
    pub fn mime(self) -> &'static str {
        match self {
            Self::Ttf => "font/ttf",
            Self::Otf => "font/otf",
            Self::Woff2 => "font/woff2",
        }
    }
    /// The CSS `@font-face` `format()` hint.
    pub fn css_format(self) -> &'static str {
        match self {
            Self::Ttf => "truetype",
            Self::Otf => "opentype",
            Self::Woff2 => "woff2",
        }
    }
    /// The file extension (for a linked font file's name).
    pub fn ext(self) -> &'static str {
        match self {
            Self::Ttf => "ttf",
            Self::Otf => "otf",
            Self::Woff2 => "woff2",
        }
    }
}

/// The weight range and slant a delivered font file covers — the `@font-face` `font-weight` /
/// `font-style` descriptors. Without these the browser treats the face as a single weight-400 upright
/// and FAUX-bolds everything (breaking cascade's bold headings); with them it engages a variable
/// font's `wght` axis, or matches the right static face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceStyle {
    /// `font-weight` as (min, max): a static face is (w, w); a variable font is its `wght` axis range.
    pub weight: (u16, u16),
    /// Italic/oblique → `font-style: italic`.
    pub italic: bool,
}

impl Default for FaceStyle {
    /// A neutral upright regular — used when the file can't be introspected (e.g. woff2).
    fn default() -> Self {
        FaceStyle { weight: (400, 400), italic: false }
    }
}

/// One delivered font face: the file + format, the weight/style it covers, and — for LINK delivery —
/// the relative href the output references (`None` = EMBED as a `data:` URI). A family is a *set* of
/// these (Regular, Bold, Italic, …), or a single face for a variable font.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub format: FontFormat,
    pub bytes: Vec<u8>,
    pub style: FaceStyle,
    /// Set for link delivery → the renderer emits `url("<href>")` and the CLI writes `bytes` there.
    pub href: Option<String>,
}

/// How a font's actual glyphs reach the medium — the DELIVERY axis, separate from the metrics.
#[derive(Clone, Debug, PartialEq)]
pub enum FontDelivery {
    /// Rely on the reader / compiler already having the family: a system font, or a bundled
    /// sane-default that falls back through the stack. No `@font-face`, no embedded file.
    System,
    /// Ship the family's faces so it renders regardless of what's installed — ONE `@font-face` per
    /// face (a static family delivers Regular + Bold + Italic + …; a variable font, one face over a
    /// weight range). The CLI reads the bytes (renderer stays pure); each face embeds as a `data:`
    /// URI, or links to a written file when it carries an `href`.
    Faces(Vec<Face>),
}

/// A font RESOLVED to its metrics + identity + delivery — everything a renderer needs, decoupled from
/// WHERE the font came from. A bundled catalog [`Font`] projects into one (`From<Font>`); a runtime
/// font (a web/system face loaded from a cached RON or measured live) produces the SAME shape. So the
/// render path is provenance-blind: it reads metrics off a *loaded font*, never off the closed enum.
/// The metric fields are the optical formulas' inputs; [`delivery`](Self::delivery) is how the glyphs
/// reach the medium (renderer-owned) — the two are kept separate.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedFont {
    /// Family name — the identity a renderer names in its output.
    pub family: String,
    /// Category — drives a renderer's generic fallback (serif / sans / mono).
    pub category: Category,
    pub optical_size: String,
    pub x_height: f64,
    pub cap_height: f64,
    pub avg_advance: f64,
    pub k_tracking: f64,
    pub leading_base: f64,
    pub word_space: f64,
    /// How to deliver the glyphs. Bundled fonts default to [`FontDelivery::System`]; an external font
    /// with an accessible file is [`FontDelivery::Embed`]ed by the CLI so it actually renders.
    pub delivery: FontDelivery,
}

impl From<Font> for ResolvedFont {
    /// Project a bundled catalog font into its resolved metrics — the COMPILE-TIME source of a
    /// [`ResolvedFont`]. A runtime external font yields the same shape from a measurement, so both
    /// feed render identically. Bundled fonts default to `System` delivery (they fall back).
    fn from(f: Font) -> Self {
        ResolvedFont {
            family: f.family().to_string(),
            category: f.category(),
            optical_size: f.optical_size().to_string(),
            x_height: f.x_height(),
            cap_height: f.cap_height(),
            avg_advance: f.avg_advance(),
            k_tracking: f.k_tracking(),
            leading_base: f.leading_base(),
            word_space: f.word_space(),
            delivery: FontDelivery::System,
        }
    }
}

/// The LIMITED surface a consumer may change per build: which modular [`ScalePreset`], the body /
/// heading / code typefaces, and the colour [`Theme`] (palette). Everything else is the spec (compiled,
/// not tunable). This is the single input to [`Renderer::render`], so every target projects the
/// SAME choices — the CLI resolves a `cascade.ron` (or flags) into one of these and hands it to each
/// renderer. `body` / `heading` / `code` are RESOLVED fonts (metrics, not the enum): a bundled [`Font`]
/// via `.into()`, or a runtime-measured external font — the renderer can't tell which. [`Default`] is
/// the spec's compiled defaults (`SCALE_DEFAULT`, `FONT_BODY`, `FONT_HEADING`, `FONT_CODE`, `THEME_DEFAULT`).
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub scale: ScalePreset,
    pub body: ResolvedFont,
    pub heading: ResolvedFont,
    pub code: ResolvedFont,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: SCALE_DEFAULT,
            body: FONT_BODY.into(),
            heading: FONT_HEADING.into(),
            code: FONT_CODE.into(),
            theme: THEME_DEFAULT,
        }
    }
}

pub trait Renderer {
    /// Short target name, e.g. `"css"` / `"typst"`.
    fn name(&self) -> &'static str;

    /// Resolve a font into this target's family representation — the renderer's FALLBACK contract.
    /// It gets the resolved font (`font.family`, `font.category`) and produces the target form:
    /// cascade-css builds a stack, cascade-typst uses the family alone. Works the same for a bundled
    /// or a runtime font — it only reads the identity, not the provenance.
    fn font_family(&self, font: &ResolvedFont) -> String;

    /// Produce all of this target's output files for the given [`Config`].
    fn render(&self, cfg: &Config) -> Vec<Output>;

    /// Verify rendered output is correct FOR THIS MEDIUM, using that medium's OWN tooling — CSS
    /// parses it with a browser-grade parser and checks for dangling references; a print target
    /// compiles it. Returns the problems found (empty = OK).
    ///
    /// This is a REQUIRED obligation of the contract, deliberately WITHOUT a default: a renderer may
    /// not ship output it cannot stand behind, so every target must supply a real check with the
    /// right tool — a no-op won't compile as one. Drives `cascade build --verify`.
    fn verify(&self, outputs: &[Output]) -> Vec<String>;
}
