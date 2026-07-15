//! Compiles the spec (tokens.ron) INTO Rust types. Every closed set in the spec becomes
//! an enum whose variants are the defined values, with data accessors — so consumers are
//! handed values valid by construction. serde/ron are used here (build-time only) to READ
//! the spec; the emitted types are plain Rust with no runtime deps.
use serde::Deserialize;
use std::fmt::Write as _;
use std::{env, fs, path::Path};

#[derive(Deserialize)]
struct Spec {
    scale_default: String,
    scale_steps: MinMaxI,
    scale_presets: Vec<Preset>,
    optical: Optical,
    category_defaults: Vec<CatDefault>,
    colors: Vec<ColorDef>,
    multipliers: Vec<Mult>,
}
#[derive(Deserialize)]
struct MinMaxI {
    min: i32,
    max: i32,
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
    units_per_em: u32,
    sx: String,
    asc: String,
    desc: String,
}

/// kebab / plain name → PascalCase Rust variant identifier.
fn pascal(s: &str) -> String {
    s.split('-')
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

fn main() {
    println!("cargo:rerun-if-changed=tokens.ron");
    println!("cargo:rerun-if-changed=fonts");
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let s: Spec = ron::from_str(&fs::read_to_string(Path::new(&dir).join("tokens.ron")).unwrap())
        .expect("parse tokens.ron");

    // One RON per typeface under fonts/ — drop in a file, it becomes a valid `Font`.
    // Sorted by name for a deterministic enum (directory order isn't guaranteed).
    let mut fonts: Vec<FontDef> = fs::read_dir(Path::new(&dir).join("fonts"))
        .expect("read fonts/")
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension().and_then(|e| e.to_str()) == Some("ron")).then(|| {
                ron::from_str(&fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
            })
        })
        .collect();
    fonts.sort_by(|a: &FontDef, b: &FontDef| a.name.cmp(&b.name));

    let mut o = String::from("// GENERATED from tokens.ron by build.rs — do not edit by hand.\n\n");

    // ── Category (+ its default optical profile: the generic, no-font-selected baseline) ──
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
    close(&mut o);

    // ── Font ──
    let fv: Vec<String> = fonts.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Font", &fv);
    let m = |g: fn(&FontDef) -> String| fonts.iter().map(g).collect::<Vec<_>>();
    method(&mut o, "family", "&'static str", &fv, &m(|x| q(&x.name)));
    method(&mut o, "category", "Category", &fv, &m(|x| cat(x.category)));
    method(&mut o, "optical_size", "&'static str", &fv, &m(|x| q(&x.profile.optical_size)));
    method(&mut o, "x_height", "f64", &fv, &m(|x| f(x.measured.x_height)));
    method(&mut o, "k_tracking", "f64", &fv, &m(|x| f(x.profile.k_tracking)));
    method(&mut o, "leading_base", "f64", &fv, &m(|x| f(x.profile.leading_base)));
    method(&mut o, "word_space", "f64", &fv, &m(|x| f(x.profile.word_space)));
    method(&mut o, "cap_height", "f64", &fv, &m(|x| f(x.measured.cap_height)));
    method(&mut o, "units_per_em", "u32", &fv, &m(|x| x.measured.units_per_em.to_string()));
    method(&mut o, "sx", "&'static str", &fv, &m(|x| q(&x.measured.sx)));
    method(&mut o, "asc", "&'static str", &fv, &m(|x| q(&x.measured.asc)));
    method(&mut o, "desc", "&'static str", &fv, &m(|x| q(&x.measured.desc)));
    close(&mut o);

    // ── ScalePreset ──
    let pv: Vec<String> = s.scale_presets.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "ScalePreset", &pv);
    method(&mut o, "id", "&'static str", &pv, &s.scale_presets.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "ratio", "f64", &pv, &s.scale_presets.iter().map(|x| f(x.ratio)).collect::<Vec<_>>());
    method(&mut o, "n", "u32", &pv, &s.scale_presets.iter().map(|x| x.n.to_string()).collect::<Vec<_>>());
    close(&mut o);

    // ── Color ──
    let cv: Vec<String> = s.colors.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Color", &cv);
    method(&mut o, "id", "&'static str", &cv, &s.colors.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "light", "&'static str", &cv, &s.colors.iter().map(|x| q(&x.light)).collect::<Vec<_>>());
    method(&mut o, "dark", "&'static str", &cv, &s.colors.iter().map(|x| q(&x.dark)).collect::<Vec<_>>());
    close(&mut o);

    // ── Multiplier ──
    let mv: Vec<String> = s.multipliers.iter().map(|x| pascal(&x.name)).collect();
    enum_head(&mut o, "Multiplier", &mv);
    method(&mut o, "id", "&'static str", &mv, &s.multipliers.iter().map(|x| q(&x.name)).collect::<Vec<_>>());
    method(&mut o, "factor", "f64", &mv, &s.multipliers.iter().map(|x| f(x.value)).collect::<Vec<_>>());
    close(&mut o);

    // ── scalar globals (renderer-agnostic only) ──
    let _ = writeln!(o, "pub const SCALE_DEFAULT: ScalePreset = ScalePreset::{};", pascal(&s.scale_default));
    let _ = writeln!(o, "pub const STEPS_MIN: i32 = {};", s.scale_steps.min);
    let _ = writeln!(o, "pub const STEPS_MAX: i32 = {};", s.scale_steps.max);
    let _ = writeln!(o, "pub const WORD_SPACE_K: f64 = {};", f(s.optical.word_space_k));
    let _ = writeln!(o, "pub const TRACKING_CLAMP: f64 = {};", f(s.optical.tracking_clamp));
    let _ = writeln!(o, "pub const LEADING_CLAMP: (f64, f64) = ({}, {});", f(s.optical.leading_clamp.min), f(s.optical.leading_clamp.max));
    let _ = writeln!(o, "pub const MEASURE: u32 = {};", s.optical.measure);

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("spec.rs"), o).unwrap();
}
