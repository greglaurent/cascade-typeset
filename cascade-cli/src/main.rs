//! cascade — the command-line composition root.
//!
//! Three jobs, each a subcommand:
//!   • `build`   — resolve the consumer's limited surface (a `cascade.ron` and/or flags) into a
//!                 [`Config`], drive a renderer, and WRITE the output files. The only place output
//!                 touches disk.
//!   • `measure` — read a font file's OS/2 / head metrics and emit a `fonts/<name>.ron`. The spec
//!                 compiles that RON into a new `Font` on the next build (drop-in + recompile).
//!   • `list`    — print the exposed surface: the scales and fonts a consumer may actually pick.
//!
//! The renderers stay pure (spec in, `Output`s out); this crate owns I/O and the outside world.
use std::collections::HashSet;
use std::path::PathBuf;

use cascade::renderer::{Config, Renderer};
use cascade::{Category, Font, ScalePreset, Theme};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cascade", about = "Type-driven typography: one spec, projected per renderer.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render the spec to a target's output files.
    Build(Build),
    /// Measure a font file → a fonts/<name>.ron the spec compiles in.
    Measure(Measure),
    /// List the surface a consumer may change (scales, fonts, targets).
    List,
}

#[derive(Args)]
struct Build {
    /// Output directory (created if missing).
    #[arg(long, default_value = "dist")]
    out: PathBuf,
    /// Render target.
    #[arg(long, default_value = "css")]
    target: String,
    /// Consumer config to read (.toml or .json). Flags below override individual fields.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Modular scale preset id (overrides --config).
    #[arg(long)]
    scale: Option<String>,
    /// Body typeface family (overrides --config).
    #[arg(long)]
    body: Option<String>,
    /// Heading typeface family (overrides --config).
    #[arg(long)]
    heading: Option<String>,
    /// Default colour palette id (overrides --config). CSS ships all palettes and switches at
    /// runtime via [data-palette]; this picks the :root default. Print bakes it. (`cascade list`)
    #[arg(long)]
    theme: Option<String>,
    /// Verify the output for its medium before writing (CSS: valid + no dangling var()s). On a
    /// problem, nothing is written and the command exits non-zero.
    #[arg(long)]
    verify: bool,
}

#[derive(Args)]
struct Measure {
    /// Font file to measure (.ttf / .otf).
    font: PathBuf,
    /// Name for the emitted Font (default: the font's family name, else the filename).
    #[arg(long)]
    name: Option<String>,
    /// Category — seeds the optical profile (serif | sans | mono).
    #[arg(long, default_value = "sans")]
    category: String,
    /// Where to write the RON (default: cascade/fonts/<name>.ron).
    #[arg(long)]
    out: Option<PathBuf>,
}

/// cascade's record of what it last wrote to an out dir — the basis for a safe, targeted clean.
/// Lives at `<out>/.cascade-manifest.json` (JSON so it's readable with standard tools).
const MANIFEST: &str = ".cascade-manifest.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Manifest {
    files: Vec<FileStamp>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct FileStamp {
    path: String,
    checksum: String,
}

/// A small, stable content hash (FNV-1a, 64-bit) — enough to detect "changed since we wrote it".
/// Not cryptographic: it guards against clobbering a user's edits, not against tampering.
fn checksum(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// The consumer's config (e.g. `cascade.toml`): every field optional, each naming a value from the
/// compiled set. A mainstream format on purpose — see [`load_file_config`].
#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    #[serde(default)]
    scale: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    heading: Option<String>,
    #[serde(default)]
    theme: Option<String>,
}

/// Read a consumer config, picking the parser from the file extension. TOML and JSON are the
/// supported formats — familiar to author, unlike the spec's internal ron. Unknown fields error
/// (`deny_unknown_fields`), so a typo'd key is caught rather than silently ignored.
fn load_file_config(path: &std::path::Path) -> Result<FileConfig, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let parse_err = |e: String| format!("parse {}: {e}", path.display());
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => toml::from_str(&text).map_err(|e| parse_err(e.to_string())),
        Some("json") => serde_json::from_str(&text).map_err(|e| parse_err(e.to_string())),
        Some(other) => Err(format!("unsupported config format '.{other}' — use .toml or .json")),
        None => Err(format!("{}: config needs a .toml or .json extension", path.display())),
    }
}

type Res = Result<(), Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprintln!("cascade: {e}");
        std::process::exit(1);
    }
}

fn run() -> Res {
    match Cli::parse().cmd {
        Cmd::Build(b) => build(b),
        Cmd::Measure(m) => measure(m),
        Cmd::List => list(),
    }
}

// ── build ──────────────────────────────────────────────────────────────────────────────────
fn build(b: Build) -> Res {
    let file = match &b.config {
        Some(p) => load_file_config(p)?,
        None => FileConfig::default(),
    };
    let cfg = resolve(file, b.scale, b.body, b.heading, b.theme)?;
    let renderer = renderer_for(&b.target)?;

    let outputs = renderer.render(&cfg);

    // Verify before writing, so broken output never lands on disk.
    if b.verify {
        let problems = renderer.verify(&outputs);
        if !problems.is_empty() {
            let mut msg = format!("{} output failed verification ({} problem(s)):", renderer.name(), problems.len());
            for p in &problems {
                msg.push_str(&format!("\n  - {p}"));
            }
            return Err(msg.into());
        }
        println!("verified: {} {} file(s) — valid CSS, no dangling references", renderer.name(), outputs.len());
    }

    std::fs::create_dir_all(&b.out)?;

    // Clean via the MANIFEST of what we last wrote. We only ever consider removing files the
    // manifest lists — ones we KNOW cascade generated — and only when they're now stale (not
    // re-emitted) AND still byte-for-byte what we wrote (checksum matches). A file we never
    // generated is untouched; a generated file the user has since edited is KEPT, never deleted.
    let manifest_path = b.out.join(MANIFEST);
    let previous: Manifest = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let fresh: HashSet<&str> = outputs.iter().map(|o| o.path.as_str()).collect();

    let (mut cleaned, mut kept) = (Vec::new(), Vec::new());
    for entry in &previous.files {
        if fresh.contains(entry.path.as_str()) {
            continue; // still emitted — it'll be overwritten below
        }
        let path = b.out.join(&entry.path);
        match std::fs::read(&path) {
            Ok(bytes) if checksum(&bytes) == entry.checksum => {
                std::fs::remove_file(&path)?;
                cleaned.push(entry.path.clone());
            }
            Ok(_) => kept.push(entry.path.clone()), // edited since generated — leave it
            Err(_) => {}                            // already gone
        }
    }

    // Write the fresh output and record the new manifest (name + checksum of exactly these bytes).
    let mut manifest = Manifest::default();
    for o in &outputs {
        let path = b.out.join(&o.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &o.body)?;
        manifest.files.push(FileStamp { path: o.path.clone(), checksum: checksum(o.body.as_bytes()) });
    }
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    if !cleaned.is_empty() {
        println!("cleaned {} stale file(s): {}", cleaned.len(), cleaned.join(", "));
    }
    if !kept.is_empty() {
        println!("kept {} file(s) edited since generated (not removed): {}", kept.len(), kept.join(", "));
    }
    println!(
        "wrote {} file(s) to {} — target: {}, scale: {}, body: {}, heading: {}, theme: {}",
        outputs.len(),
        b.out.display(),
        renderer.name(),
        cfg.scale.id(),
        cfg.body.family(),
        cfg.heading.family(),
        cfg.theme.id(),
    );
    Ok(())
}

/// Overlay flags onto the file onto the compiled defaults; resolve every name against its closed
/// set (an unknown value lists what IS available, rather than a silent fallback).
fn resolve(
    file: FileConfig,
    scale: Option<String>,
    body: Option<String>,
    heading: Option<String>,
    theme: Option<String>,
) -> Result<Config, String> {
    let mut cfg = Config::default();
    for s in file.scale.iter().chain(scale.iter()) {
        cfg.scale = parse_scale(s)?;
    }
    for f in file.body.iter().chain(body.iter()) {
        cfg.body = parse_font(f)?;
    }
    for f in file.heading.iter().chain(heading.iter()) {
        cfg.heading = parse_font(f)?;
    }
    for t in file.theme.iter().chain(theme.iter()) {
        cfg.theme = parse_theme(t)?;
    }
    Ok(cfg)
}

fn parse_scale(s: &str) -> Result<ScalePreset, String> {
    ScalePreset::from_id(s).ok_or_else(|| {
        let all = ScalePreset::ALL.iter().map(|p| p.id()).collect::<Vec<_>>().join(", ");
        format!("unknown scale '{s}'. available: {all}")
    })
}

fn parse_font(s: &str) -> Result<Font, String> {
    Font::from_family(s).ok_or_else(|| {
        let all = Font::ALL.iter().map(|f| f.family()).collect::<Vec<_>>().join(", ");
        format!("unknown font '{s}'. available: {all}")
    })
}

fn parse_theme(s: &str) -> Result<Theme, String> {
    Theme::from_id(s).ok_or_else(|| {
        let all = Theme::ALL.iter().map(|t| t.id()).collect::<Vec<_>>().join(", ");
        format!("unknown theme '{s}'. available: {all}")
    })
}

fn renderer_for(target: &str) -> Result<Box<dyn Renderer>, String> {
    match target {
        "css" => Ok(Box::new(cascade_css::Css)),
        // TODO(typst): add `"typst" => Ok(Box::new(cascade_typst::Typst))` once its render/verify
        // are implemented (they're todo!() stubs today), and add cascade-typst as a dependency.
        other => Err(format!("unknown target '{other}'. available: css")),
    }
}

// ── measure ────────────────────────────────────────────────────────────────────────────────
fn measure(m: Measure) -> Res {
    let cat = Category::from_str(&m.category).ok_or_else(|| {
        format!("unknown category '{}'. available: serif, sans, mono", m.category)
    })?;
    let data = std::fs::read(&m.font).map_err(|e| format!("read {}: {e}", m.font.display()))?;
    let face = ttf_parser::Face::parse(&data, 0).map_err(|e| format!("parse font: {e}"))?;

    let upem = face.units_per_em() as f64;
    // x-height is the whole point of measuring: raw OS/2 sxHeight ÷ units_per_em. No sxHeight → we
    // cannot normalize this font's optical, so refuse rather than invent a value.
    let sx = face
        .x_height()
        .ok_or("font has no OS/2 x-height (sxHeight); cannot measure")? as f64;
    // cap height: OS/2 sCapHeight if present, else the 'H' glyph's top.
    let cap = face
        .capital_height()
        .map(|v| v as f64)
        .or_else(|| face.glyph_index('H').and_then(|g| face.glyph_bounding_box(g)).map(|b| b.y_max as f64))
        .ok_or("font has no cap height (sCapHeight, no 'H' glyph)")?;
    let asc = face.typographic_ascender().unwrap_or_else(|| face.ascender()) as f64;
    let desc = face.typographic_descender().unwrap_or_else(|| face.descender()) as f64;

    let name = m
        .name
        .or_else(|| family_name(&face))
        .or_else(|| m.font.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .ok_or("could not determine a font name; pass --name")?;
    let slug = name.to_lowercase().replace(' ', "-");

    // profile = the category's generic optical baseline; the author tunes from there.
    let ron = format!(
        "// cascade — font measure: {name} ({cat}). OS/2 metrics + optical profile.\n\
         (\n    \
         name: {name:?},\n    \
         category: {cat},\n    \
         profile:  (optical_size: {os:?}, k_tracking: {kt}, leading_base: {lb}, word_space: {ws}),\n    \
         measured: (x_height: {xh:.3}, cap_height: {ch:.3}, units_per_em: {upem_i}, sx: \"sxHeight {sx_i}\", asc: {asc:?}, desc: {desc:?}),\n\
         )\n",
        cat = cat.as_str(),
        os = cat.default_optical_size(),
        kt = fnum(cat.default_k_tracking()),
        lb = fnum(cat.default_leading_base()),
        ws = fnum(cat.default_word_space()),
        xh = sx / upem,
        ch = cap / upem,
        upem_i = upem as u32,
        sx_i = sx as i64,
        asc = format!("{:.3}", asc / upem),
        desc = format!("{:.3}", desc / upem),
    );

    let out = m.out.unwrap_or_else(|| PathBuf::from(format!("cascade/fonts/{slug}.ron")));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, &ron)?;
    println!(
        "measured {} → {} (x-height {:.3}, cap {:.3}, upem {})",
        name,
        out.display(),
        sx / upem,
        cap / upem,
        upem as u32,
    );
    if out.starts_with("cascade/fonts") || out.components().any(|c| c.as_os_str() == "fonts") {
        println!("  the spec picks this up on the next build (drop-in + recompile).");
    }
    Ok(())
}

/// The font's own family name (name id 1), if present and decodable.
fn family_name(face: &ttf_parser::Face) -> Option<String> {
    face.names().into_iter().find(|n| n.name_id == 1).and_then(|n| n.to_string())
}

/// f64 → a RON-safe decimal literal (always a decimal point, so a whole number stays an f64).
fn fnum(v: f64) -> String {
    format!("{v:?}")
}

// ── list ───────────────────────────────────────────────────────────────────────────────────
fn list() -> Res {
    let d = Config::default();
    println!("scales (--scale):");
    for p in ScalePreset::ALL {
        let tag = if p == d.scale { "  (default)" } else { "" };
        println!("  {}{}", p.id(), tag);
    }
    println!("\nfonts (--body / --heading):");
    for f in Font::ALL {
        let mut tags = Vec::new();
        if f == d.body {
            tags.push("body-default");
        }
        if f == d.heading {
            tags.push("heading-default");
        }
        let tag = if tags.is_empty() { String::new() } else { format!("  ({})", tags.join(", ")) };
        println!("  {:8} {}{}", f.family(), f.category().as_str(), tag);
    }
    println!("\nthemes (--theme):");
    for t in Theme::ALL {
        let tag = if t == d.theme { "  (default)" } else { "" };
        println!("  {}{}", t.id(), tag);
    }
    println!("\ntargets (--target):\n  css  (default)");
    Ok(())
}
