//! Renderers: build each renderer's askama context from the spec/manifest, then render.
//!
//! Each template has its OWN context struct exposing only the fields that renderer
//! supports — so a template physically cannot reference a feature its target lacks
//! (that would be a compile error). Formatting/alignment is computed here in Rust; the
//! templates just lay it out. Output is byte-identical to the legacy `just gen`.
use crate::manifest::Manifest;
use crate::spec::Spec;
use askama::Template;

/// Provenance stamp baked into every generated file's header. (Still references the
/// legacy source string so M2 output is byte-identical to the current files; the string
/// updates when `just gen` flips to the Rust path in M4.)
const GEN: &str = "GENERATED from tokens.mjs by `just gen` — do not edit by hand";

// ── cascade-typst/scale.typ ─────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "typst/scale.typ", escape = "none")]
struct ScaleTypst<'a> {
    banner: &'a str,
    base_print: &'a str,
    default_ratio: f64,
    default_n: u32,
    steps: Vec<StepRow>,
    presets: Vec<ScalePresetRow>,
}

struct StepRow {
    k: String,
    i: i32,
}

struct ScalePresetRow {
    padded_name: String,
    ratio: f64,
    n: u32,
}

/// Render `cascade-typst/scale.typ` from the spec.
pub fn scale_typ(spec: &Spec) -> String {
    let def = spec
        .scale
        .presets
        .iter()
        .find(|p| p.name == spec.scale.default)
        .expect("scale.default resolves to a preset (validated)");

    let steps = (spec.scale.steps.min..=spec.scale.steps.max)
        .map(|i| {
            let k = if i == 0 {
                "base".to_string()
            } else if i < 0 {
                format!("n{}", -i)
            } else {
                format!("p{i}")
            };
            StepRow { k, i }
        })
        .collect();

    // Column width: the mjs used `(name + ':').padEnd(maxNameLen + 1)`.
    let w = spec.scale.presets.iter().map(|p| p.name.len() + 1).max().unwrap_or(0);
    let presets = spec
        .scale
        .presets
        .iter()
        .map(|p| ScalePresetRow {
            padded_name: format!("{:width$}", format!("{}:", p.name), width = w),
            ratio: p.ratio,
            n: p.n,
        })
        .collect();

    ScaleTypst {
        banner: GEN,
        base_print: &spec.scale.base.print,
        default_ratio: def.ratio,
        default_n: def.n,
        steps,
        presets,
    }
    .render()
    .expect("render scale.typ")
}

// ── cascade-typst/rhythm.typ ────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "typst/rhythm.typ", escape = "none")]
struct RhythmTypst<'a> {
    banner: &'a str,
    scale_default: &'a str,
    measure: u32,
    grid_unit_print: &'a str,
    multipliers: Vec<MultRow>,
}

struct MultRow {
    padded_name: String,
    value: f64,
}

/// Render `cascade-typst/rhythm.typ` from the spec.
pub fn rhythm_typ(spec: &Spec) -> String {
    // The mjs padded the spacing keys to a fixed width of 6.
    let multipliers = spec
        .rhythm
        .multipliers
        .iter()
        .map(|m| MultRow {
            padded_name: format!("{:6}", format!("{}:", m.name)),
            value: m.value,
        })
        .collect();

    RhythmTypst {
        banner: GEN,
        scale_default: &spec.scale.default,
        measure: spec.optical.measure,
        grid_unit_print: &spec.rhythm.unit.print,
        multipliers,
    }
    .render()
    .expect("render rhythm.typ")
}

// ── cascade-typst/theme.typ ─────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "typst/theme.typ", escape = "none")]
struct ThemeTypst<'a> {
    banner: &'a str,
    light: Vec<ColorRow>,
    dark: Vec<ColorRow>,
}

struct ColorRow {
    padded_name: String,
    hex: String,
}

/// The mjs padded color keys to a fixed width of 16.
fn color_rows(colors: &[crate::spec::Color]) -> Vec<ColorRow> {
    colors
        .iter()
        .map(|c| ColorRow {
            padded_name: format!("{:16}", format!("{}:", c.name)),
            hex: c.hex.clone(),
        })
        .collect()
}

/// Render `cascade-typst/theme.typ` from the spec.
pub fn theme_typ(spec: &Spec) -> String {
    ThemeTypst {
        banner: GEN,
        light: color_rows(&spec.theme.light),
        dark: color_rows(&spec.theme.dark),
    }
    .render()
    .expect("render theme.typ")
}

// ── cascade-typst/font.typ ──────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "typst/font.typ", escape = "none")]
struct FontTypst<'a> {
    banner: &'a str,
    leading_clamp_min: f64,
    leading_clamp_max: f64,
    tracking_clamp: f64,
    word_space_k: f64,
    measure: u32,
    scale_default: &'a str,
    profiles: Vec<ProfileRow>,
    fonts: Vec<ProfileRow>,
    generic_bundles: Vec<BundleRow>,
    font_bundles: Vec<BundleRow>,
}

struct ProfileRow {
    padded_name: String,
    make: String,
}

struct BundleRow {
    name: String,
    family: String,
    profile_key: String,
}

/// The `make(...)` call for one profile — Typst uses `optical-size` (a Typst-only field
/// the CSS renderer reinterprets as the body base). Column-padded keys handled by caller.
fn mk_typst(optical_size: &str, x_height: f64, k_tracking: f64, leading_base: f64, word_space: f64) -> String {
    format!(
        "make(optical-size: {optical_size}, x-height: {x_height}, k-tracking: {k_tracking}, leading-base: {leading_base}, base-word-space: {word_space})"
    )
}

/// Render `cascade-typst/font.typ` from the spec (optical model + generic bundles) and
/// the manifest (the packaged typefaces' profiles + bundles).
pub fn font_typ(spec: &Spec, manifest: &Manifest) -> String {
    // Generic optical profiles (spec) and font-specific profiles (manifest). Fixed pad 12.
    let profiles = spec
        .optical
        .profiles
        .iter()
        .map(|p| ProfileRow {
            padded_name: format!("{:12}", format!("{}:", p.name)),
            make: mk_typst(&p.optical_size, p.x_height, p.k_tracking, p.leading_base, p.word_space),
        })
        .collect();
    let fonts = manifest
        .fonts
        .iter()
        .map(|f| ProfileRow {
            padded_name: format!("{:12}", format!("{}:", f.name)),
            make: mk_typst(&f.profile.optical_size, f.profile.x_height, f.profile.k_tracking, f.profile.leading_base, f.profile.word_space),
        })
        .collect();

    // Generic bundles carry the stack's Typst family; font bundles carry the typeface's.
    let stack_family = |name: &str| {
        spec.generics
            .stacks
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.typst.clone())
            .expect("bundle → stack resolves (validated)")
    };
    let generic_bundles = spec
        .generics
        .bundles
        .iter()
        .map(|b| BundleRow {
            name: b.name.clone(),
            family: stack_family(&b.stack),
            profile_key: b.profile.clone(),
        })
        .collect();
    let font_bundles = manifest
        .fonts
        .iter()
        .map(|f| BundleRow {
            name: f.name.clone(),
            family: f.family.typst.clone(),
            profile_key: f.name.clone(),
        })
        .collect();

    FontTypst {
        banner: GEN,
        leading_clamp_min: spec.optical.leading_clamp.min,
        leading_clamp_max: spec.optical.leading_clamp.max,
        tracking_clamp: spec.optical.tracking_clamp,
        word_space_k: spec.optical.word_space_k,
        measure: spec.optical.measure,
        scale_default: &spec.scale.default,
        profiles,
        fonts,
        generic_bundles,
        font_bundles,
    }
    .render()
    .expect("render font.typ")
}

// ════════════════════════════════════════════════════════════════════════════════
//  CSS renderer
//  Note: none of these contexts carry `optical_size` — CSS reinterprets it as the body
//  base, so the CSS templates *cannot* reference it (it isn't in scope). That's the
//  per-renderer feature boundary made structural.
// ════════════════════════════════════════════════════════════════════════════════

// ── cascade-css/scale.css ───────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "css/scale.css", escape = "none")]
struct ScaleCss<'a> {
    banner: &'a str,
    base_web: &'a str,
    default_name: &'a str,
    default_ratio: f64,
    default_n: u32,
    neg: Vec<CssSizeRow>,
    pos: Vec<CssSizeRow>,
    presets: Vec<CssScalePreset>,
}

struct CssSizeRow {
    label: String,
    num: String,
}

struct CssScalePreset {
    name: String,
    ratio: f64,
    n: u32,
}

/// Render `cascade-css/scale.css` from the spec.
pub fn scale_css(spec: &Spec) -> String {
    let def = spec
        .scale
        .presets
        .iter()
        .find(|p| p.name == spec.scale.default)
        .expect("scale.default resolves (validated)");

    // Steps split around the base (step 0), which is a literal line. `num` mirrors the
    // mjs `String(i).padStart(2)` — negatives keep the sign, positives get a leading space.
    let label = |i: i32| if i < 0 { format!("n{}", -i) } else { format!("p{i}") };
    let neg = (spec.scale.steps.min..0)
        .map(|i| CssSizeRow { label: label(i), num: format!("{i:2}") })
        .collect();
    let pos = (1..=spec.scale.steps.max)
        .map(|i| CssSizeRow { label: label(i), num: format!("{i:2}") })
        .collect();
    let presets = spec
        .scale
        .presets
        .iter()
        .map(|p| CssScalePreset { name: p.name.clone(), ratio: p.ratio, n: p.n })
        .collect();

    ScaleCss {
        banner: GEN,
        base_web: &spec.scale.base.web,
        default_name: &spec.scale.default,
        default_ratio: def.ratio,
        default_n: def.n,
        neg,
        pos,
        presets,
    }
    .render()
    .expect("render scale.css")
}

// ── cascade-css/rhythm.css ──────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "css/rhythm.css", escape = "none")]
struct RhythmCss<'a> {
    banner: &'a str,
    unit_web: f64,
    spaces: Vec<CssSpaceRow>,
}

struct CssSpaceRow {
    padded_name: String,
    value: f64,
}

/// Render `cascade-css/rhythm.css` from the spec.
pub fn rhythm_css(spec: &Spec) -> String {
    // Custom-property name is `--cr-space-<key>`, where the `base` multiplier maps to 0.
    // The mjs padded `(name + ':')` to a fixed width of 14.
    let spaces = spec
        .rhythm
        .multipliers
        .iter()
        .map(|m| {
            let key = if m.name == "base" { "0" } else { m.name.as_str() };
            CssSpaceRow {
                padded_name: format!("{:14}", format!("--cr-space-{key}:")),
                value: m.value,
            }
        })
        .collect();

    RhythmCss { banner: GEN, unit_web: spec.rhythm.unit.web, spaces }
        .render()
        .expect("render rhythm.css")
}

// ── cascade-css/theme.css ───────────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "css/theme.css", escape = "none")]
struct ThemeCss<'a> {
    banner: &'a str,
    light: Vec<CssColorRow>,
    dark: Vec<CssColorRow>,
}

struct CssColorRow {
    padded_name: String,
    hex: String,
}

/// The mjs padded `--ct-<name>:` to a fixed width of 21.
fn css_color_rows(colors: &[crate::spec::Color]) -> Vec<CssColorRow> {
    colors
        .iter()
        .map(|c| CssColorRow {
            padded_name: format!("{:21}", format!("--ct-{}:", c.name)),
            hex: c.hex.clone(),
        })
        .collect()
}

/// Render `cascade-css/theme.css` from the spec.
pub fn theme_css(spec: &Spec) -> String {
    ThemeCss {
        banner: GEN,
        light: css_color_rows(&spec.theme.light),
        dark: css_color_rows(&spec.theme.dark),
    }
    .render()
    .expect("render theme.css")
}

// ── cascade-css/font.css ────────────────────────────────────────────────────────
fn find_profile<'a>(spec: &'a Spec, name: &str) -> &'a crate::spec::Profile {
    spec.optical
        .profiles
        .iter()
        .find(|p| p.name == name)
        .expect("bundle → profile resolves (validated)")
}

#[derive(Template)]
#[template(path = "css/font.css", escape = "none")]
struct FontCss<'a> {
    banner: &'a str,
    stacks: Vec<CssStackRow>,
    def_stack: &'a str,
    def_name: &'a str,
    def_xh: f64,
    def_kt: f64,
    def_lb: f64,
    def_bws: f64,
    kws: f64,
    tc: f64,
    lmin: f64,
    lmax: f64,
    measure: u32,
    size_min_web: &'a str,
    neg: Vec<CssSizeRow>,
    pos: Vec<CssSizeRow>,
    heads: Vec<u32>,
    profile_classes: Vec<ProfileClass>,
    bundle_classes: Vec<BundleClass>,
    heading_classes: Vec<HeadingClass>,
}

struct CssStackRow {
    name: String,
    css: String,
}

struct ProfileClass {
    name: String,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
}

struct BundleClass {
    name: String,
    stack: String,
    xh: f64,
    kt: f64,
    lb: f64,
    bws: f64,
}

struct HeadingClass {
    name: String,
    stack: String,
    xh: f64,
    kt: f64,
    lb: f64,
}

/// Render `cascade-css/font.css` from the spec. This is generic families + the optical
/// model only; the packaged typefaces live in the per-font `fonts/<name>.css`.
pub fn font_css(spec: &Spec) -> String {
    let stacks = spec
        .generics
        .stacks
        .iter()
        .map(|s| CssStackRow { name: s.name.clone(), css: s.css.clone() })
        .collect();

    let def_bundle = spec
        .generics
        .bundles
        .iter()
        .find(|b| b.name == spec.generics.default)
        .expect("generics.default resolves (validated)");
    let dp = find_profile(spec, &def_bundle.profile);

    let label = |i: i32| if i < 0 { format!("n{}", -i) } else { format!("p{i}") };
    let neg = (spec.scale.steps.min..0)
        .map(|i| CssSizeRow { label: label(i), num: format!("{i:2}") })
        .collect();
    let pos = (1..=spec.scale.steps.max)
        .map(|i| CssSizeRow { label: label(i), num: format!("{i:2}") })
        .collect();

    let profile_classes = spec
        .optical
        .profiles
        .iter()
        .map(|p| ProfileClass { name: p.name.clone(), xh: p.x_height, kt: p.k_tracking, lb: p.leading_base, bws: p.word_space })
        .collect();
    let bundle_classes = spec
        .generics
        .bundles
        .iter()
        .map(|b| {
            let p = find_profile(spec, &b.profile);
            BundleClass { name: b.name.clone(), stack: b.stack.clone(), xh: p.x_height, kt: p.k_tracking, lb: p.leading_base, bws: p.word_space }
        })
        .collect();
    let heading_classes = spec
        .generics
        .bundles
        .iter()
        .map(|b| {
            let p = find_profile(spec, &b.profile);
            HeadingClass { name: b.name.clone(), stack: b.stack.clone(), xh: p.x_height, kt: p.k_tracking, lb: p.leading_base }
        })
        .collect();

    FontCss {
        banner: GEN,
        stacks,
        def_stack: &def_bundle.stack,
        def_name: &spec.generics.default,
        def_xh: dp.x_height,
        def_kt: dp.k_tracking,
        def_lb: dp.leading_base,
        def_bws: dp.word_space,
        kws: spec.optical.word_space_k,
        tc: spec.optical.tracking_clamp,
        lmin: spec.optical.leading_clamp.min,
        lmax: spec.optical.leading_clamp.max,
        measure: spec.optical.measure,
        size_min_web: &spec.optical.size_min.web,
        neg,
        pos,
        heads: vec![1, 2, 3, 4],
        profile_classes,
        bundle_classes,
        heading_classes,
    }
    .render()
    .expect("render font.css")
}

// ── cascade-css/fonts/<name>.css ────────────────────────────────────────────────
#[derive(Template)]
#[template(path = "css/font_preset.css", escape = "none")]
struct FontPresetCss<'a> {
    banner: &'a str,
    name: &'a str,
    cap_name: String,
    category: &'a str,
    units_per_em: u32,
    measured_xh: f64,
    sx: &'a str,
    cap_height: f64,
    asc: &'a str,
    desc: &'a str,
    family_css: &'a str,
    p_xh: f64,
    p_kt: f64,
    p_lb: f64,
    p_bws: f64,
    normalize: Option<NormalizeCtx>,
}

struct NormalizeCtx {
    to_xh: f64,
    size_adjust: String,
    alias: String,
    family_typst: String,
    ascent: String,
    descent: String,
}

/// Capitalize the first character (the mjs `cap()`).
fn cap(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Render one packaged typeface's `cascade-css/fonts/<name>.css` from a manifest font.
pub fn font_preset_css(font: &crate::manifest::Font) -> String {
    let normalize = font.normalize.as_ref().map(|n| NormalizeCtx {
        to_xh: n.to_x_height,
        size_adjust: n.size_adjust.clone(),
        alias: n.alias.clone(),
        family_typst: font.family.typst.clone(),
        ascent: n.ascent.clone(),
        descent: n.descent.clone(),
    });

    FontPresetCss {
        banner: GEN,
        name: &font.name,
        cap_name: cap(&font.name),
        category: font.category.as_str(),
        units_per_em: font.measured.units_per_em,
        measured_xh: font.measured.x_height,
        sx: &font.measured.sx,
        cap_height: font.measured.cap_height,
        asc: &font.measured.asc,
        desc: &font.measured.desc,
        family_css: &font.family.css,
        p_xh: font.profile.x_height,
        p_kt: font.profile.k_tracking,
        p_lb: font.profile.leading_base,
        p_bws: font.profile.word_space,
        normalize,
    }
    .render()
    .expect("render font preset css")
}

// ════════════════════════════════════════════════════════════════════════════════
//  Emit
// ════════════════════════════════════════════════════════════════════════════════

/// Every generated file, as (repo-root-relative path, contents). The single list the
/// CLI writes; adding a renderer/output = one entry here.
pub fn files(spec: &Spec, manifest: &Manifest) -> Vec<(String, String)> {
    let mut out = vec![
        ("cascade-typst/scale.typ".to_string(), scale_typ(spec)),
        ("cascade-typst/font.typ".to_string(), font_typ(spec, manifest)),
        ("cascade-typst/theme.typ".to_string(), theme_typ(spec)),
        ("cascade-typst/rhythm.typ".to_string(), rhythm_typ(spec)),
        ("cascade-css/scale.css".to_string(), scale_css(spec)),
        ("cascade-css/font.css".to_string(), font_css(spec)),
        ("cascade-css/theme.css".to_string(), theme_css(spec)),
        ("cascade-css/rhythm.css".to_string(), rhythm_css(spec)),
    ];
    for f in &manifest.fonts {
        out.push((format!("cascade-css/fonts/{}.css", f.name), font_preset_css(f)));
    }
    out
}
