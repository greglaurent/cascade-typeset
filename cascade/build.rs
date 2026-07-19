//! Build-time spec generator. Reads the spec RON (tokens.ron + theme.ron + themes/ + fonts/) from
//! this crate's own directory (CARGO_MANIFEST_DIR) and writes the generated Rust types to OUT_DIR,
//! which src/spec.rs `include!`s. Because cargo sets CARGO_MANIFEST_DIR and OUT_DIR correctly for
//! EVERY build -- this crate built directly, or as a path/git/registry dependency in any workspace
//! -- generation works anywhere cascade is depended on. (This replaces a crabtime proc-macro, which
//! assumed it was the top-level project and could not run when cascade was a dependency.)

use serde::Deserialize;
use std::fmt::Write as _;

#[derive(Deserialize)]
struct Spec {
    scale_default: String,
    default_fonts: DefaultFonts,
    scale_steps: MinMaxI,
    scale_presets: Vec<Preset>,
    optical: Optical,
    category_defaults: Vec<CatDefault>,
    multipliers: Vec<Mult>,
    roles: Vec<RoleDef>,
}
// theme.ron -- the colour layer's SCHEMA: the swatch->use bindings every palette shares, plus
// which palette is default. The swatch VALUES live in themes/*.ron (one `Palette` each).
#[derive(Deserialize)]
struct Theme {
    default: String,
    semantic: Vec<SemDef>,
    roles: Vec<RoleColorDef>,
}
// themes/<name>.ron -- one palette: the swatch values (light + dark) for the shared vocabulary.
#[derive(Deserialize)]
struct Palette {
    palette: Vec<ColorDef>,
}
#[derive(Deserialize)]
struct SemDef {
    name: String,
    base: String,
}
#[derive(Deserialize)]
struct RoleColorDef {
    role: String,
    #[serde(default)]
    fg: Option<String>,
    #[serde(default)]
    bg: Option<String>,
    #[serde(default)]
    border: Option<String>,
}
#[derive(Deserialize)]
struct MinMaxI {
    min: i32,
    max: i32,
}
#[derive(Deserialize)]
struct DefaultFonts {
    body: String,
    heading: String,
    code: String,
}
#[derive(Deserialize)]
struct Preset {
    name: String,
    ratio: f64,
    n: u32,
}
#[derive(Deserialize)]
struct Optical {
    word_space_k: f64,
    tracking_clamp: f64,
    leading_clamp: MinMax,
    measure: u32,
    base_pt: f64,
    size_min_pt: f64,
    rhythm_unit_ratio: f64,
    code_scale: f64,
}
#[derive(Deserialize)]
struct MinMax {
    min: f64,
    max: f64,
}
#[derive(Deserialize)]
struct CatDefault {
    category: Cat,
    optical_size: String,
    x_height: f64,
    k_tracking: f64,
    leading_base: f64,
    word_space: f64,
    avg_advance: f64,
}
#[derive(Deserialize)]
struct ColorDef {
    name: String,
    light: String,
    dark: String,
}
#[derive(Deserialize)]
struct Mult {
    name: String,
    value: f64,
}
#[derive(Deserialize)]
struct RoleDef {
    name: String,
    elements: Vec<String>,
    kind: RKind,
    #[serde(default)]
    step: Option<i32>,
    #[serde(default)]
    font: Option<FRole>,
    #[serde(default)]
    weight: Option<u32>,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    space_before: Option<String>,
    #[serde(default)]
    space_after: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum RKind {
    Block,
    Inline,
}
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum FRole {
    Body,
    Heading,
    Code,
}
#[derive(Deserialize)]
struct FontDef {
    name: String,
    category: Cat,
    profile: Prof,
    measured: Meas,
}
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Cat {
    Serif,
    Sans,
    Mono,
}
#[derive(Deserialize)]
struct Prof {
    optical_size: String,
    k_tracking: f64,
    leading_base: f64,
    word_space: f64,
}
#[derive(Deserialize)]
struct Meas {
    x_height: f64,
    cap_height: f64,
    avg_advance: f64,
    units_per_em: u32,
    sx: String,
    asc: String,
    desc: String,
}

/// kebab / spaced / plain name -> PascalCase Rust variant identifier (e.g. "IBM Plex Mono" ->
/// "IBMPlexMono"). Splits on both '-' and ' ' so multi-word font families are valid identifiers;
/// `Font::family()` keeps the original spaced name for the CSS output.
fn pascal(s: &str) -> String {
    s.split(['-', ' '])
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
fn q(s: &str) -> String {
    format!("{s:?}")
}
fn f(x: f64) -> String {
    format!("{x:?}") // always a decimal literal (2.0, not 2)
}
fn cat(c: Cat) -> String {
    format!(
        "Category::{}",
        match c {
            Cat::Serif => "Serif",
            Cat::Sans => "Sans",
            Cat::Mono => "Mono",
        }
    )
}
fn cat_name(c: Cat) -> &'static str {
    match c {
        Cat::Serif => "serif",
        Cat::Sans => "sans",
        Cat::Mono => "mono",
    }
}

/// Emit `enum {name}` + open its `impl` with an `ALL` const.
fn enum_head(out: &mut String, name: &str, variants: &[String]) {
    let _ = writeln!(out, "#[derive(Clone, Copy, PartialEq, Eq, Debug)]\npub enum {name} {{");
    for v in variants {
        let _ = writeln!(out, "    {v},");
    }
    let all = variants.iter().map(|v| format!("{name}::{v}")).collect::<Vec<_>>().join(", ");
    let _ = writeln!(out, "}}\n\nimpl {name} {{");
    let _ = writeln!(out, "    pub const ALL: [{name}; {}] = [{all}];", variants.len());
}

/// Emit `pub fn {name}(self) -> {ret}` mapping each variant to a preformatted expr.
fn method(out: &mut String, name: &str, ret: &str, variants: &[String], exprs: &[String]) {
    let _ = writeln!(out, "    pub fn {name}(self) -> {ret} {{\n        match self {{");
    for (v, e) in variants.iter().zip(exprs) {
        let _ = writeln!(out, "            Self::{v} => {e},");
    }
    let _ = writeln!(out, "        }}\n    }}");
}
fn close(out: &mut String) {
    let _ = writeln!(out, "}}\n");
}

/// `None` / `Some(<f(v)>)` for an optional scalar accessor.
fn opt<T: Copy>(o: Option<T>, f: impl Fn(T) -> String) -> String {
    match o {
        Some(v) => format!("Some({})", f(v)),
        None => "None".into(),
    }
}

fn main() {
    // Regenerate when the spec RON changes or a font/theme is dropped in.
    println!("cargo:rerun-if-changed=tokens.ron");
    println!("cargo:rerun-if-changed=theme.ron");
    println!("cargo:rerun-if-changed=themes");
    println!("cargo:rerun-if-changed=fonts");

    // The spec RON lives in this crate's own directory -- correct whether cascade is built directly
    // or as a dependency, because cargo always sets CARGO_MANIFEST_DIR to the crate being built.
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let dir = std::path::Path::new(&dir);
    let s: Spec = ron::from_str(&std::fs::read_to_string(dir.join("tokens.ron")).unwrap())
        .expect("parse tokens.ron");
    let theme: Theme = ron::from_str(&std::fs::read_to_string(dir.join("theme.ron")).unwrap())
        .expect("parse theme.ron");

    // One RON per palette under themes/ -- drop in a file, it becomes a selectable `Theme`.
    // Sorted by name for a deterministic enum. Each file's stem is the theme's id.
    let mut themes: Vec<(String, Palette)> = std::fs::read_dir(dir.join("themes"))
        .expect("read themes/")
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension().and_then(|e| e.to_str()) == Some("ron")).then(|| {
                let name = path.file_stem().unwrap().to_string_lossy().into_owned();
                let pal: Palette = ron::from_str(&std::fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
                (name, pal)
            })
        })
        .collect();
    themes.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!themes.is_empty(), "no palettes in themes/ -- need at least one themes/<name>.ron");
    assert!(
        themes.iter().any(|(name, _)| name == &theme.default),
        "theme.ron default '{}' is not a palette in themes/",
        theme.default
    );

    // The default palette fixes the swatch vocabulary + `Color` variant order; every other palette
    // must define exactly the same swatch names (so `Color` is well-defined for all `Theme`s).
    let default_pal = &themes.iter().find(|(n, _)| n == &theme.default).unwrap().1;
    let swatch_order: Vec<&str> = default_pal.palette.iter().map(|c| c.name.as_str()).collect();
    let swatch_set: std::collections::HashSet<&str> = swatch_order.iter().copied().collect();
    for (name, pal) in &themes {
        let names: std::collections::HashSet<&str> = pal.palette.iter().map(|c| c.name.as_str()).collect();
        assert!(
            names == swatch_set,
            "palette '{name}' swatches differ from default '{}': every themes/*.ron must define the same swatch names",
            theme.default
        );
    }
    // Per-theme lookup: swatch name -> (light, dark).
    let swatch = |t: &Palette, name: &str, dark: bool| -> String {
        let c = t.palette.iter().find(|c| c.name == name).unwrap();
        q(if dark { &c.dark } else { &c.light })
    };

    // One RON per typeface under fonts/ -- drop in a file, it becomes a valid `Font`.
    // Sorted by name for a deterministic enum (directory order isn't guaranteed).
    let mut fonts: Vec<FontDef> = std::fs::read_dir(dir.join("fonts"))
        .expect("read fonts/")
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension().and_then(|e| e.to_str()) == Some("ron")).then(|| {
                ron::from_str(&std::fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
            })
        })
        .collect();
    fonts.sort_by(|a: &FontDef, b: &FontDef| a.name.cmp(&b.name));

    let mut o = String::from("// GENERATED by build.rs from tokens.ron/theme.ron/themes/fonts -- do not edit by hand.\n\n");

    // -- Category (+ its default optical profile: the generic, no-font-selected baseline) --
    let cats: Vec<String> = vec!["Serif".into(), "Sans".into(), "Mono".into()];
    let order = ["serif", "sans", "mono"];
    let by_cat = |want: &str| {
        s.category_defaults
            .iter()
            .find(|d| cat_name(d.category) == want)
            .unwrap_or_else(|| panic!("missing category_default for {want}"))
    };
    let cd = |g: fn(&CatDefault) -> String| order.iter().map(|&c| g(by_cat(c))).collect::<Vec<_>>();
    enum_head(&mut o, "Category", &cats);
    method(&mut o, "as_str", "&'static str", &cats, &[q("serif"), q("sans"), q("mono")]);
    method(&mut o, "default_optical_size", "&'static str", &cats, &cd(|d| q(&d.optical_size)));
    method(&mut o, "default_x_height", "f64", &cats, &cd(|d| f(d.x_height)));
    method(&mut o, "default_k_tracking", "f64", &cats, &cd(|d| f(d.k_tracking)));
    method(&mut o, "default_leading_base", "f64", &cats, &cd(|d| f(d.leading_base)));
    method(&mut o, "default_word_space", "f64", &cats, &cd(|d| f(d.word_space)));
    method(&mut o, "default_avg_advance", "f64", &cats, &cd(|d| f(d.avg_advance)));
    close(&mut o);

    // -- Font --
    let fv: Vec<String> = fonts.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Font", &fv);
    let m = |g: fn(&FontDef) -> String| fonts.iter().map(g).collect::<Vec<_>>();
    method(&mut o, "family", "&'static str", &fv, &m(|x| q(&x.name)));
    method(&mut o, "category", "Category", &fv, &m(|x| cat(x.category)));
    method(&mut o, "optical_size", "&'static str", &fv, &m(|x| q(&x.profile.optical_size)));
    method(&mut o, "x_height", "f64", &fv, &m(|x| f(x.measured.x_height)));
    method(&mut o, "avg_advance", "f64", &fv, &m(|x| f(x.measured.avg_advance)));
    method(&mut o, "k_tracking", "f64", &fv, &m(|x| f(x.profile.k_tracking)));
    method(&mut o, "leading_base", "f64", &fv, &m(|x| f(x.profile.leading_base)));
    method(&mut o, "word_space", "f64", &fv, &m(|x| f(x.profile.word_space)));
    method(&mut o, "cap_height", "f64", &fv, &m(|x| f(x.measured.cap_height)));
    method(&mut o, "units_per_em", "u32", &fv, &m(|x| x.measured.units_per_em.to_string()));
    method(&mut o, "sx", "&'static str", &fv, &m(|x| q(&x.measured.sx)));
    method(&mut o, "asc", "&'static str", &fv, &m(|x| q(&x.measured.asc)));
    method(&mut o, "desc", "&'static str", &fv, &m(|x| q(&x.measured.desc)));
    close(&mut o);

    // -- Font::source_bytes: the vendored source file (fonts/sources/<slug>.ttf), EMBEDDED, for
    // delivering a bundled font where the medium needs the actual file (Typst compiles against font
    // files, not names). Feature-gated behind `measure` so type-only consumers don't carry the binary.
    // include_bytes! makes each .ttf a compile dependency, so a source change rebuilds automatically. --
    let _ = writeln!(o, "#[cfg(feature = \"measure\")]\nimpl Font {{");
    let _ = writeln!(o, "    /// The vendored source font bytes, embedded — for delivery where the medium needs the");
    let _ = writeln!(o, "    /// actual file (e.g. Typst). Feature-gated (`measure`).");
    let _ = writeln!(o, "    pub fn source_bytes(self) -> &'static [u8] {{\n        match self {{");
    for x in &fonts {
        let slug: String = x.name.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let _ = writeln!(
            o,
            "            Self::{} => include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/fonts/sources/{slug}.ttf\")),",
            pascal(&x.name),
        );
    }
    let _ = writeln!(o, "        }}\n    }}\n}}\n");

    // -- ScalePreset --
    let pv: Vec<String> = s.scale_presets.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "ScalePreset", &pv);
    method(&mut o, "id", "&'static str", &pv, &s.scale_presets.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "ratio", "f64", &pv, &s.scale_presets.iter().map(|x| f(x.ratio)).collect::<Vec<_>>());
    method(&mut o, "n", "u32", &pv, &s.scale_presets.iter().map(|x| x.n.to_string()).collect::<Vec<_>>());
    close(&mut o);

    // -- Color (the swatch VOCABULARY -- names only; the values are per-`Theme`) --
    let cv: Vec<String> = swatch_order.iter().map(|n| pascal(n)).collect();
    enum_head(&mut o, "Color", &cv);
    method(&mut o, "id", "&'static str", &cv, &swatch_order.iter().map(|n| q(n)).collect::<Vec<_>>());
    close(&mut o);

    // -- Theme (a palette). `light`/`dark` resolve a `Color`'s value FOR THIS palette; both ship,
    // so the renderer emits every mode and the browser picks live within the selected theme. --
    let tv: Vec<String> = themes.iter().map(|(n, _)| pascal(n)).collect();
    enum_head(&mut o, "Theme", &tv);
    method(&mut o, "id", "&'static str", &tv, &themes.iter().map(|(n, _)| q(n)).collect::<Vec<_>>());
    for (fname, want_dark) in [("light", false), ("dark", true)] {
        let _ = writeln!(o, "    pub fn {fname}(self, c: Color) -> &'static str {{\n        match self {{");
        for (tname, pal) in &themes {
            let _ = writeln!(o, "            Self::{} => match c {{", pascal(tname));
            for sw in &swatch_order {
                let _ = writeln!(o, "                Color::{} => {},", pascal(sw), swatch(pal, sw, want_dark));
            }
            let _ = writeln!(o, "            }},");
        }
        let _ = writeln!(o, "        }}\n    }}");
    }
    close(&mut o);

    // -- Semantic (named colour uses; each aliases a base swatch or the `transparent` literal) --
    // Validate every alias target against the palette at build time -- an unknown swatch won't compile.
    let palette_names: std::collections::HashSet<&str> = swatch_order.iter().copied().collect();
    for sem in &theme.semantic {
        assert!(
            palette_names.contains(sem.base.as_str()) || sem.base == "transparent",
            "theme semantic '{}' aliases unknown colour '{}'",
            sem.name, sem.base
        );
    }
    let sv: Vec<String> = theme.semantic.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Semantic", &sv);
    method(&mut o, "id", "&'static str", &sv, &theme.semantic.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "base", "&'static str", &sv, &theme.semantic.iter().map(|x| q(&x.base)).collect::<Vec<_>>());
    close(&mut o);

    // -- role_color: the role -> colour binding (THE composition point). Keyed by the structural
    // Role, but NOT a method on it -- Role stays colour-free. Validated at build time. --
    let role_names: std::collections::HashSet<&str> = s.roles.iter().map(|r| r.name.as_str()).collect();
    let semantic_names: std::collections::HashSet<&str> =
        theme.semantic.iter().map(|x| x.name.as_str()).collect();
    let color_ref = |slot: &Option<String>, role: &str| match slot {
        None => "None".to_string(),
        Some(name) => {
            assert!(
                palette_names.contains(name.as_str())
                    || semantic_names.contains(name.as_str())
                    || name == "transparent",
                "role_color '{role}' references unknown colour '{name}'"
            );
            format!("Some({})", q(name))
        }
    };
    let _ = writeln!(
        o,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub struct RoleColor {{ pub fg: Option<&'static str>, pub bg: Option<&'static str>, pub border: Option<&'static str> }}\n"
    );
    let _ = writeln!(o, "/// The theme's colour binding for a role -- `None` slots inherit. Renderer combines this");
    let _ = writeln!(o, "/// with the structural [`Role`]; the palette/semantic names resolve to the target's colour form.");
    let _ = writeln!(o, "pub fn role_color(role: Role) -> RoleColor {{\n    match role {{");
    for rc in &theme.roles {
        assert!(role_names.contains(rc.role.as_str()), "role_color binds unknown role '{}'", rc.role);
        let _ = writeln!(
            o,
            "        Role::{} => RoleColor {{ fg: {}, bg: {}, border: {} }},",
            pascal(&rc.role),
            color_ref(&rc.fg, &rc.role),
            color_ref(&rc.bg, &rc.role),
            color_ref(&rc.border, &rc.role),
        );
    }
    let _ = writeln!(o, "        _ => RoleColor {{ fg: None, bg: None, border: None }},\n    }}\n}}\n");

    // -- Multiplier --
    let mv: Vec<String> = s.multipliers.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Multiplier", &mv);
    method(&mut o, "id", "&'static str", &mv, &s.multipliers.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "factor", "f64", &mv, &s.multipliers.iter().map(|x| f(x.value)).collect::<Vec<_>>());
    close(&mut o);

    // -- RoleKind + FontRole (the fixed vocabularies a role references) --
    let rk = vec!["Block".to_string(), "Inline".to_string()];
    enum_head(&mut o, "RoleKind", &rk);
    method(&mut o, "as_str", "&'static str", &rk, &[q("block"), q("inline")]);
    close(&mut o);
    let fr = vec!["Body".to_string(), "Heading".to_string(), "Code".to_string()];
    enum_head(&mut o, "FontRole", &fr);
    method(&mut o, "as_str", "&'static str", &fr, &[q("body"), q("heading"), q("code")]);
    close(&mut o);

    // -- Role (the document model) --
    let frole = |v: FRole| {
        format!("FontRole::{}", match v {
            FRole::Body => "Body",
            FRole::Heading => "Heading",
            FRole::Code => "Code",
        })
    };
    let rv: Vec<String> = s.roles.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Role", &rv);
    let rm = |g: fn(&RoleDef) -> String| s.roles.iter().map(g).collect::<Vec<_>>();
    method(&mut o, "id", "&'static str", &rv, &rm(|x| q(&x.name)));
    method(&mut o, "elements", "&'static [&'static str]", &rv, &s.roles.iter()
        .map(|x| format!("&[{}]", x.elements.iter().map(|e| q(e)).collect::<Vec<_>>().join(", ")))
        .collect::<Vec<_>>());
    method(&mut o, "kind", "RoleKind", &rv, &rm(|x| format!("RoleKind::{}", match x.kind {
        RKind::Block => "Block",
        RKind::Inline => "Inline",
    })));
    method(&mut o, "step", "Option<i32>", &rv, &rm(|x| opt(x.step, |v| v.to_string())));
    method(&mut o, "font", "Option<FontRole>", &rv, &s.roles.iter()
        .map(|x| opt(x.font, frole)).collect::<Vec<_>>());
    method(&mut o, "weight", "Option<u32>", &rv, &rm(|x| opt(x.weight, |v| v.to_string())));
    method(&mut o, "italic", "bool", &rv, &rm(|x| x.italic.to_string()));
    method(&mut o, "underline", "bool", &rv, &rm(|x| x.underline.to_string()));
    method(&mut o, "space_before", "Option<&'static str>", &rv, &rm(|x| opt(x.space_before.as_deref(), q)));
    method(&mut o, "space_after", "Option<&'static str>", &rv, &rm(|x| opt(x.space_after.as_deref(), q)));
    close(&mut o);

    // -- scalar globals (renderer-agnostic only) --
    for (role, name) in [
        ("body", &s.default_fonts.body),
        ("heading", &s.default_fonts.heading),
        ("code", &s.default_fonts.code),
    ] {
        assert!(fonts.iter().any(|f| &f.name == name), "default_fonts.{role} '{name}' is not a font in fonts/");
    }
    let _ = writeln!(o, "pub const FONT_BODY: Font = Font::{};", pascal(&s.default_fonts.body));
    let _ = writeln!(o, "pub const FONT_HEADING: Font = Font::{};", pascal(&s.default_fonts.heading));
    let _ = writeln!(o, "pub const FONT_CODE: Font = Font::{};", pascal(&s.default_fonts.code));
    let _ = writeln!(o, "pub const SCALE_DEFAULT: ScalePreset = ScalePreset::{};", pascal(&s.scale_default));
    let _ = writeln!(o, "pub const THEME_DEFAULT: Theme = Theme::{};", pascal(&theme.default));
    let _ = writeln!(o, "pub const STEPS_MIN: i32 = {};", s.scale_steps.min);
    let _ = writeln!(o, "pub const STEPS_MAX: i32 = {};", s.scale_steps.max);
    let _ = writeln!(o, "pub const WORD_SPACE_K: f64 = {};", f(s.optical.word_space_k));
    let _ = writeln!(o, "pub const TRACKING_CLAMP: f64 = {};", f(s.optical.tracking_clamp));
    let _ = writeln!(o, "pub const LEADING_CLAMP: (f64, f64) = ({}, {});", f(s.optical.leading_clamp.min), f(s.optical.leading_clamp.max));
    let _ = writeln!(o, "pub const MEASURE: u32 = {};", s.optical.measure);
    let _ = writeln!(o, "pub const BASE_PT: f64 = {};", f(s.optical.base_pt));
    let _ = writeln!(o, "pub const SIZE_MIN_PT: f64 = {};", f(s.optical.size_min_pt));
    let _ = writeln!(o, "pub const RHYTHM_UNIT_RATIO: f64 = {};", f(s.optical.rhythm_unit_ratio));
    let _ = writeln!(o, "pub const CODE_SCALE: f64 = {};", f(s.optical.code_scale));

    let out = std::env::var("OUT_DIR").expect("OUT_DIR");
    std::fs::write(std::path::Path::new(&out).join("spec.rs"), o).expect("write generated spec.rs");
}
