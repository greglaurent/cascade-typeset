//! Font measurement — the STANDARDIZED derivation of a font's optical metrics from its OpenType
//! tables, and the `fonts/<name>.ron` emission format the spec compiles. This is the single source:
//! the `cascade measure` CLI just reads/writes files and calls in here, so "how a font is measured"
//! lives in the spec, next to what the numbers mean.
//!
//! Feature-gated behind `measure` (adds the `ttf-parser` dep) so the spec's core stays
//! dependency-free: a consumer that only wants the generated types never pulls a font reader.
use crate::renderer::{FaceStyle, FontDelivery, ResolvedFont};
use crate::Category;
use ttf_parser::Face;

/// Frequency of each character in English running text (letters by standard corpus frequency, plus
/// the word space at ~18%). Weights sum to ~1; [`measure_face`] renormalises over whatever glyphs
/// the font actually provides, so a missing glyph just drops from the average.
pub const CHAR_FREQ: &[(char, f64)] = &[
    (' ', 0.1828), ('e', 0.1041), ('t', 0.0729), ('a', 0.0651), ('o', 0.0616), ('i', 0.0567),
    ('n', 0.0559), ('s', 0.0537), ('r', 0.0499), ('h', 0.0428), ('l', 0.0331), ('d', 0.0328),
    ('c', 0.0237), ('u', 0.0227), ('m', 0.0202), ('f', 0.0198), ('p', 0.0180), ('g', 0.0160),
    ('w', 0.0154), ('y', 0.0152), ('b', 0.0126), ('v', 0.0080), ('k', 0.0056), ('x', 0.0014),
    ('j', 0.0010), ('q', 0.0008), ('z', 0.0005),
];

/// A font's measured optical facts — exactly the `measured:` block of a `fonts/<name>.ron`. Pure
/// font data (no tuning): `measure` regenerates this; the author-tuned `profile` is preserved
/// separately. Ratios are em fractions (÷ units_per_em) unless noted.
#[derive(Clone, Debug)]
pub struct Measured {
    /// The font's own family name (OpenType name id 1), if present and decodable.
    pub family: Option<String>,
    pub x_height: f64,
    pub cap_height: f64,
    /// Frequency-weighted mean character advance — the copyfitting factor turning a character
    /// `measure` into a real reading width (cf. OS/2 `xAvgCharWidth` / Capsize `xWidthAvg`).
    pub avg_advance: f64,
    pub units_per_em: u32,
    /// Raw OS/2 sxHeight, in font units — kept for the RON's human-readable note.
    pub sx_height_units: i64,
    pub ascender: f64,
    pub descender: f64,
}

/// The font's sfnt (TrueType/OpenType) bytes — decompressing woff2 (Brotli + table reconstruction)
/// so ttf-parser can read it; every other format passes through untouched. woff2 is the ubiquitous
/// web font format, so measuring and introspecting it is first-class, not a dead end.
fn sfnt(data: &[u8]) -> Result<std::borrow::Cow<'_, [u8]>, String> {
    if woff2::decode::is_woff2(data) {
        woff2::decode::convert_woff2_to_ttf(&mut std::io::Cursor::new(data.to_vec()))
            .map(std::borrow::Cow::Owned)
            .map_err(|e| format!("woff2 decode: {e:?}"))
    } else {
        Ok(std::borrow::Cow::Borrowed(data))
    }
}

/// Measure a font from its raw bytes (a .ttf/.otf, or a woff2 which is decompressed first): the
/// em-normalized optical facts, or an error if the font lacks a metric cascade requires. x-height is
/// the whole point — no OS/2 sxHeight → we cannot normalize the optical, so we refuse rather than
/// invent one.
pub fn measure_face(data: &[u8]) -> Result<Measured, String> {
    let data = sfnt(data)?;
    let mut face = Face::parse(&data, 0).map_err(|e| format!("parse font: {e}"))?;
    let upem = face.units_per_em() as f64;

    // opsz is a CHOICE, not an average: pin the optical-size axis to its text end (min) so we
    // measure the reading face, not a display cut — opsz moves x-height AND advance materially
    // (Inter x-height 0.546@14 → 0.516@32). Averaging across opsz would blend text with display.
    if let Some(a) = variation_axis(&face, b"opsz") {
        let _ = face.set_variation(ttf_parser::Tag::from_bytes(b"opsz"), a.min_value);
    }

    // wght IS averaged: the RON holds one metric vector but the advance thickens with weight and the
    // body may be set at any weight. Sample the full metric vector at k weight positions → matrix M
    // (k × metrics) and left-multiply by the weight vector p (`m* = pᵀ M`), p peaked at the reading
    // (default) weight. Weight-INVARIANT rows of M (x-height, cap, asc, desc) fall out as their own
    // constant; the advance row averages non-trivially — one uniform op, no per-metric special-case.
    let m = match variation_axis(&face, b"wght") {
        Some(a) => {
            let wght = ttf_parser::Tag::from_bytes(b"wght");
            let mut acc = MetricVec::ZERO;
            for (pos, p) in wght_prior(a.min_value, a.def_value, a.max_value) {
                let _ = face.set_variation(wght, pos);
                acc = acc.add_scaled(measure_at(&face, upem)?, p);
            }
            let _ = face.set_variation(wght, a.def_value); // restore the default instance
            acc
        }
        None => measure_at(&face, upem)?, // static font: one row, no averaging
    };

    Ok(Measured {
        family: face.names().into_iter().find(|n| n.name_id == 1).and_then(|n| n.to_string()),
        x_height: m.x_height,
        cap_height: m.cap_height,
        avg_advance: m.avg_advance,
        units_per_em: upem as u32,
        sx_height_units: (m.x_height * upem).round() as i64,
        ascender: m.ascender,
        descender: m.descender,
    })
}

/// The weight-axis sampling: how many positions to sample (`k`, the rows of `M`), and the shape of
/// the prior `p`. The prior is a Gaussian peaked at the DEFAULT (reading) weight but ASYMMETRIC —
/// a wider half-width on the bold side than the light side, because body text runs regular→bold far
/// more often than regular→light. Half-widths are fractions of each side's span (0 = the default,
/// 1 = that end of the axis). Tunable in one place.
const WGHT_SAMPLES: usize = 9;
const SIGMA_LIGHT: f64 = 0.4; // narrow: light body is rare
const SIGMA_BOLD: f64 = 0.8; // wide: medium/semibold/bold body is common

/// The averaging prior `p`: `WGHT_SAMPLES` positions evenly across `[min, max]`, each weighted by the
/// asymmetric Gaussian above and normalized to `Σp = 1`. Returns `(weight-axis position, p_i)`.
fn wght_prior(min: f32, def: f32, max: f32) -> Vec<(f32, f64)> {
    let (min, def, max) = (min as f64, def as f64, max as f64);
    let mut samples: Vec<(f32, f64)> = (0..WGHT_SAMPLES)
        .map(|i| {
            let t = i as f64 / (WGHT_SAMPLES - 1) as f64;
            let pos = min + (max - min) * t;
            let (span, sigma) =
                if pos >= def { ((max - def).max(1.0), SIGMA_BOLD) } else { ((def - min).max(1.0), SIGMA_LIGHT) };
            let rel = (pos - def).abs() / span;
            (pos as f32, (-(rel * rel) / (2.0 * sigma * sigma)).exp())
        })
        .collect();
    let sum: f64 = samples.iter().map(|(_, w)| w).sum();
    for s in &mut samples {
        s.1 /= sum;
    }
    samples
}

/// One row of the metric matrix `M`: the em-normalized optical facts at the face's CURRENT variation
/// coordinates. Weight-invariant across the wght axis except `avg_advance`.
#[derive(Clone, Copy)]
struct MetricVec {
    x_height: f64,
    cap_height: f64,
    avg_advance: f64,
    ascender: f64,
    descender: f64,
}

impl MetricVec {
    const ZERO: MetricVec =
        MetricVec { x_height: 0.0, cap_height: 0.0, avg_advance: 0.0, ascender: 0.0, descender: 0.0 };
    /// `self + p · other` — one term of the weighted sum `pᵀ M`.
    fn add_scaled(self, o: MetricVec, p: f64) -> MetricVec {
        MetricVec {
            x_height: self.x_height + o.x_height * p,
            cap_height: self.cap_height + o.cap_height * p,
            avg_advance: self.avg_advance + o.avg_advance * p,
            ascender: self.ascender + o.ascender * p,
            descender: self.descender + o.descender * p,
        }
    }
}

/// Measure the em-normalized metric vector at the face's current variation state. Errors if the font
/// lacks the metrics cascade requires (OS/2 x-height, a cap height) — we refuse rather than invent.
fn measure_at(face: &Face, upem: f64) -> Result<MetricVec, String> {
    let sx = face.x_height().ok_or("font has no OS/2 x-height (sxHeight); cannot measure")? as f64;
    // cap height: OS/2 sCapHeight if present, else the 'H' glyph's top.
    let cap = face
        .capital_height()
        .map(|v| v as f64)
        .or_else(|| face.glyph_index('H').and_then(|g| face.glyph_bounding_box(g)).map(|b| b.y_max as f64))
        .ok_or("font has no cap height (sCapHeight, no 'H' glyph)")?;
    let asc = face.typographic_ascender().unwrap_or_else(|| face.ascender()) as f64;
    let desc = face.typographic_descender().unwrap_or_else(|| face.descender()) as f64;
    Ok(MetricVec {
        x_height: sx / upem,
        cap_height: cap / upem,
        avg_advance: weighted_advance(face) / upem,
        ascender: asc / upem,
        descender: desc / upem,
    })
}

/// A variable-font axis by tag (`b"wght"`, `b"opsz"`), or `None` for a static font / absent axis.
fn variation_axis(face: &Face, tag: &[u8; 4]) -> Option<ttf_parser::VariationAxis> {
    let t = ttf_parser::Tag::from_bytes(tag);
    face.variation_axes().into_iter().find(|a| a.tag == t)
}

/// The frequency-weighted mean glyph advance, in font units (divide by units_per_em for an em
/// fraction). Each glyph's horizontal advance is weighted by how often that character occurs in
/// English prose, so the result is the average width of a *typical* character of running text.
fn weighted_advance(face: &Face) -> f64 {
    let (mut num, mut den) = (0.0, 0.0);
    for &(ch, w) in CHAR_FREQ {
        if let Some(adv) = face.glyph_index(ch).and_then(|g| face.glyph_hor_advance(g)) {
            num += w * adv as f64;
            den += w;
        }
    }
    if den > 0.0 {
        num / den
    } else {
        0.5 * face.units_per_em() as f64 // degenerate fallback: the old textbook average
    }
}

/// f64 → a RON-safe decimal literal (always a decimal point, so a whole number stays an f64).
fn fnum(v: f64) -> String {
    format!("{v:?}")
}

/// The category-seeded `profile` block: the generic optical baseline used on a FIRST measure, which
/// the author then tunes. Re-measuring an existing font preserves its tuned profile instead — see
/// [`extract_profile`].
pub fn default_profile(category: Category) -> String {
    format!(
        "(optical_size: {os:?}, k_tracking: {kt}, leading_base: {lb}, word_space: {ws})",
        os = category.default_optical_size(),
        kt = fnum(category.default_k_tracking()),
        lb = fnum(category.default_leading_base()),
        ws = fnum(category.default_word_space()),
    )
}

/// Extract the `profile` block verbatim from an existing font RON, so re-measuring keeps the author's
/// tuning. Returns the `(...)` text (no trailing comma), or `None` if there's no profile line.
pub fn extract_profile(existing_ron: &str) -> Option<String> {
    field(existing_ron, "profile:")
}

/// Extract the `category` (`serif`/`sans`/`mono`) from an existing font RON, so a re-measure keeps
/// it without re-specifying `--category`. `None` if absent.
pub fn extract_category(existing_ron: &str) -> Option<String> {
    field(existing_ron, "category:")
}

/// Extract the `name` from an existing font RON (quotes stripped), so a re-measure keeps the authored
/// name rather than the font's raw family string (e.g. Jost ships family `"Jost*"`). `None` if absent.
pub fn extract_name(existing_ron: &str) -> Option<String> {
    field(existing_ron, "name:").map(|s| s.trim_matches('"').to_string())
}

/// The value of a top-level RON field line (`key: value,`), trimmed, trailing comma removed.
fn field(ron: &str, key: &str) -> Option<String> {
    ron.lines()
        .find_map(|l| l.trim().strip_prefix(key).map(|rest| rest.trim().trim_end_matches(',').to_string()))
}

// ── loading a measured font RON back into a ResolvedFont (the runtime, external-font path) ──
// The inverse of `font_ron`: read a `<name>.ron` (bundled OR measured by the client for a web/system
// face) into the same shape render consumes. So an external font — measured separately, cached as a
// RON — feeds the pipeline identically to a bundled one, keyed by its family name.
#[derive(serde::Deserialize)]
struct FontRon {
    name: String,
    category: RonCat,
    profile: RonProfile,
    measured: RonMeasured,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum RonCat {
    Serif,
    Sans,
    Mono,
}
#[derive(serde::Deserialize)]
struct RonProfile {
    optical_size: String,
    k_tracking: f64,
    leading_base: f64,
    word_space: f64,
}
#[derive(serde::Deserialize)]
struct RonMeasured {
    x_height: f64,
    cap_height: f64,
    avg_advance: f64,
    #[allow(dead_code)]
    units_per_em: u32,
    #[allow(dead_code)]
    sx: String,
    #[allow(dead_code)]
    asc: String,
    #[allow(dead_code)]
    desc: String,
}

/// Parse a font RON (the format [`font_ron`] emits) into a [`ResolvedFont`] — for loading an external
/// font the client measured separately, so it resolves by family name exactly like a bundled one.
pub fn load_ron(text: &str) -> Result<ResolvedFont, String> {
    let f: FontRon = ron::from_str(text).map_err(|e| format!("parse font RON: {e}"))?;
    Ok(ResolvedFont {
        family: f.name,
        category: match f.category {
            RonCat::Serif => Category::Serif,
            RonCat::Sans => Category::Sans,
            RonCat::Mono => Category::Mono,
        },
        optical_size: f.profile.optical_size,
        x_height: f.measured.x_height,
        cap_height: f.measured.cap_height,
        avg_advance: f.measured.avg_advance,
        k_tracking: f.profile.k_tracking,
        leading_base: f.profile.leading_base,
        word_space: f.profile.word_space,
        // A RON carries metrics only; the CLI augments delivery if a font file sits beside it.
        delivery: FontDelivery::System,
    })
}

/// Measure a font file's bytes straight into a [`ResolvedFont`] — for on-the-fly use (build given a
/// raw `.ttf`/`.otf` via `--font-path`, no separate `cascade measure` step). Metrics come from the
/// font; the optical `profile` comes from the CATEGORY defaults (there's no tuned profile without a
/// RON). `name` overrides the family; `category` is the caller's (build defaults it to sans).
pub fn resolve_face(data: &[u8], name: Option<&str>, category: Category) -> Result<ResolvedFont, String> {
    let m = measure_face(data)?;
    Ok(ResolvedFont {
        family: name.map(str::to_string).or(m.family).ok_or("could not determine a font name")?,
        category,
        optical_size: category.default_optical_size().to_string(),
        x_height: m.x_height,
        cap_height: m.cap_height,
        avg_advance: m.avg_advance,
        k_tracking: category.default_k_tracking(),
        leading_base: category.default_leading_base(),
        word_space: category.default_word_space(),
        delivery: FontDelivery::System, // the CLI fills Embed with these same bytes
    })
}

/// A font file's identity + style, for grouping a family's weight/style files together. `family` is
/// the PREFERRED family (OpenType name id 16, else the legacy id 1) — so weight-split files like
/// "Source Serif 4 Bold" group under "Source Serif 4".
#[derive(Clone, Debug)]
pub struct FaceInfo {
    pub family: Option<String>,
    pub style: FaceStyle,
}

/// The weight range + slant of an already-parsed face — a variable font's `wght` axis range (so the
/// axis engages for bold), else its static `usWeightClass`; italic from OS/2.
fn style_of(face: &Face) -> FaceStyle {
    let weight = face
        .tables()
        .fvar
        .and_then(|fvar| fvar.axes.into_iter().find(|a| a.tag == ttf_parser::Tag::from_bytes(b"wght")))
        .map(|a| (a.min_value as u16, a.max_value as u16))
        .unwrap_or_else(|| {
            let w = face.weight().to_number();
            (w, w)
        });
    FaceStyle { weight, italic: face.is_italic() }
}

/// Read a font file's weight range + slant for `@font-face` descriptors (woff2 decompressed first).
/// Returns the neutral default if the bytes can't be read.
pub fn face_style(data: &[u8]) -> FaceStyle {
    sfnt(data).ok().and_then(|b| Face::parse(&b, 0).ok().map(|f| style_of(&f))).unwrap_or_default()
}

/// Read a font file's preferred family + style — for grouping a family's files and describing each
/// (woff2 decompressed first).
pub fn face_info(data: &[u8]) -> FaceInfo {
    let default = || FaceInfo { family: None, style: FaceStyle::default() };
    let Ok(bytes) = sfnt(data) else { return default() };
    let Ok(face) = Face::parse(&bytes, 0) else { return default() };
    let name = |id: u16| face.names().into_iter().find(|n| n.name_id == id).and_then(|n| n.to_string());
    FaceInfo { family: name(16).or_else(|| name(1)), style: style_of(&face) }
}

/// Render a complete `fonts/<name>.ron` — the canonical on-disk format `build.rs` compiles. The
/// `measured` block is regenerated from `m`; `profile` is passed in (preserved or category-seeded).
pub fn font_ron(name: &str, category: Category, profile: &str, m: &Measured) -> String {
    format!(
        "// cascade — font measure: {name} ({cat}). OS/2 metrics + optical profile.\n\
         (\n    \
         name: {name:?},\n    \
         category: {cat},\n    \
         profile:  {profile},\n    \
         measured: (x_height: {xh:.3}, cap_height: {ch:.3}, avg_advance: {aa:.4}, units_per_em: {upem}, sx: \"sxHeight {sx}\", asc: {asc:?}, desc: {desc:?}),\n\
         )\n",
        cat = category.as_str(),
        xh = m.x_height,
        ch = m.cap_height,
        aa = m.avg_advance,
        upem = m.units_per_em,
        sx = m.sx_height_units,
        asc = format!("{:.3}", m.ascender),
        desc = format!("{:.3}", m.descender),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_freq_is_a_normalized_distribution() {
        let sum: f64 = CHAR_FREQ.iter().map(|&(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 0.02, "CHAR_FREQ weights should sum to ~1, got {sum}");
        assert!(CHAR_FREQ.iter().any(|&(c, _)| c == ' ')); // the word space is counted
    }

    #[test]
    fn ron_format_round_trips_and_seeds_or_preserves_profile() {
        let m = Measured {
            family: Some("Demo".into()),
            x_height: 0.5,
            cap_height: 0.7,
            avg_advance: 0.4635,
            units_per_em: 1000,
            sx_height_units: 500,
            ascender: 1.006,
            descender: -0.274,
        };
        // seeded profile (first measure) comes from the category defaults
        let seeded = default_profile(Category::Serif);
        let ron = font_ron("Demo", Category::Serif, &seeded, &m);
        assert!(ron.contains("avg_advance: 0.4635"));
        assert!(ron.contains("asc: \"1.006\", desc: \"-0.274\""));
        // a tuned profile is preserved verbatim across a re-measure
        let tuned = extract_profile(&ron).unwrap();
        assert_eq!(tuned, seeded);
        let ron2 = font_ron("Demo", Category::Serif, &tuned, &m);
        assert_eq!(ron, ron2, "re-emitting with the extracted profile must be idempotent");
    }

    #[test]
    fn load_ron_is_the_inverse_of_font_ron() {
        // A RON written by `font_ron` loads back into a ResolvedFont with the render-relevant fields
        // — so an external font (measured separately, cached as a RON) resolves like a bundled one.
        let m = Measured {
            family: Some("Demo".into()),
            x_height: 0.5,
            cap_height: 0.7,
            avg_advance: 0.4635,
            units_per_em: 1000,
            sx_height_units: 500,
            ascender: 1.006,
            descender: -0.274,
        };
        let profile = default_profile(Category::Serif); // optical_size "11pt", leading_base 1.35, …
        let rf = load_ron(&font_ron("Demo", Category::Serif, &profile, &m)).unwrap();
        assert_eq!(rf.family, "Demo");
        assert_eq!(rf.category, Category::Serif);
        assert_eq!(rf.x_height, 0.5);
        assert_eq!(rf.avg_advance, 0.4635);
        assert_eq!(rf.optical_size, "11pt");
        assert_eq!(rf.leading_base, 1.35); // carried from the profile block
    }
}
