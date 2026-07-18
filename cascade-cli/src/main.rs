//! cascade — the command-line composition root.
//!
//! The subcommands (this crate + cargo + build.rs are the whole toolchain — there is no task runner):
//!   • `build`   — resolve the consumer's limited surface (a `cascade.ron` and/or flags) into a
//!                 [`Config`], drive a renderer, and WRITE the output files. The only place output
//!                 touches disk.
//!   • `dist`    — the release step: build the committed distribution (verified, VERSION-stamped,
//!                 no manifest) into `dist/css`.
//!   • `measure` — read a font's OS/2 / head metrics (via [`cascade::measure`]) and emit a
//!                 `fonts/<name>.ron` the spec compiles into a `Font` (drop-in + recompile). Takes a
//!                 single font or a directory (re-measure the whole shipped set in one call).
//!   • `list`    — print the exposed surface: the scales and fonts a consumer may actually pick.
//!
//! The renderers stay pure (spec in, `Output`s out); this crate owns I/O and the outside world.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use cascade::renderer::{Config, Face, FaceStyle, FontDelivery, FontFormat, Renderer, ResolvedFont};
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
    /// Build the SHIPPED distribution (verified, VERSION-stamped, no manifest). Cascade's release step.
    Dist(Dist),
    /// Measure a font file (or a directory of them) → fonts/<name>.ron the spec compiles in.
    Measure(Measure),
    /// Fetch an EXTERNAL family from Google Fonts (its files + license + a measured RON) so `build
    /// --font-path` can use it. Standalone: adds to disk, doesn't build.
    Add(Add),
    /// List the surface a consumer may change (scales, fonts, targets).
    List,
}

#[derive(Args)]
struct Dist {
    /// Output directory for the committed distribution (created if missing). Each renderer's output
    /// lands in a `<format>/` subdir (`dist/css`, `dist/typst`).
    #[arg(long, default_value = "dist")]
    out: PathBuf,
}

#[derive(Args)]
struct Add {
    /// Family to fetch from Google Fonts, e.g. "Source Serif 4".
    family: String,
    /// Directory to add the family into (a `<slug>/` subdir is created). Point `build --font-path` here.
    #[arg(long, default_value = "fonts")]
    out: PathBuf,
    /// google/fonts git ref (branch, tag, or commit) — pin a commit for reproducibility.
    #[arg(long = "ref", default_value = "main")]
    git_ref: String,
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
    /// Code / monospace typeface family (overrides --config).
    #[arg(long)]
    code: Option<String>,
    /// Provision EXTERNAL fonts (repeatable): a font file (.ttf/.otf, measured on the fly), a measured
    /// .ron, or a directory of either. Each becomes selectable by family name via --body/--heading,
    /// like a bundled font. A font file beside a .ron is embedded for delivery. Adds to --config
    /// `font_paths`. (On-the-fly fonts default to the sans category; `cascade measure --category`
    /// first for serif/mono precision.)
    #[arg(long)]
    font_path: Vec<PathBuf>,
    /// Deliver external fonts as SEPARATE files (`<out>/fonts/<name>.<ext>`) referenced by url(),
    /// instead of base64-embedding them in the CSS — lighter output for large fonts.
    #[arg(long)]
    link_fonts: bool,
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
    /// A font file (.ttf / .otf — both read identically) OR a directory of them (measures each).
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
    code: Option<String>,
    #[serde(default)]
    theme: Option<String>,
    /// External font provisioning: paths to measured font RONs (or directories of them). The
    /// file-based twin of `--font-path`; the two are merged.
    #[serde(default)]
    font_paths: Vec<String>,
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
        Cmd::Dist(d) => dist(d),
        Cmd::Measure(m) => measure(m),
        Cmd::Add(a) => add(a),
        Cmd::List => list(),
    }
}

// ── build ──────────────────────────────────────────────────────────────────────────────────
fn build(b: Build) -> Res {
    let file = match &b.config {
        Some(p) => load_file_config(p)?,
        None => FileConfig::default(),
    };
    // Provision external fonts from config `font_paths` + `--font-path` flags (merged), then resolve
    // --body/--heading names against the bundled catalog AND these.
    let paths: Vec<PathBuf> =
        file.font_paths.iter().map(PathBuf::from).chain(b.font_path.iter().cloned()).collect();
    let mut external = load_external(&paths)?;
    // --link-fonts: deliver as a separate file (url) instead of a base64 blob. Remap each embeddable
    // external font's delivery to Link with a relative href the renderer references and we write below.
    if b.link_fonts {
        for f in &mut external {
            let slug = slugify(&f.family);
            if let FontDelivery::Faces(faces) = &mut f.delivery {
                for face in faces.iter_mut() {
                    face.href = Some(format!("fonts/{}.{}", face_stem(&slug, face.style), face.format.ext()));
                }
            }
        }
    }
    let mut cfg = resolve(file, b.scale, b.body, b.heading, b.code, b.theme, &external)?;

    // Typst compiles against font FILES, not names — so for the typst target EVERY selected font must
    // be delivered as a written file (referenced via the font path), not left `System`. A bundled font
    // pulls in its EMBEDDED source (carries with the spec, works anywhere — no repo-relative faces dir);
    // an external one already carries bytes. CSS keeps its System/embed/link behaviour untouched.
    if b.target == "typst" {
        for f in [&mut cfg.body, &mut cfg.heading, &mut cfg.code] {
            if matches!(f.delivery, FontDelivery::System)
                && let Some(font) = Font::from_family(&f.family)
            {
                let bytes = font.source_bytes().to_vec();
                let style = cascade::measure::face_style(&bytes);
                f.delivery = FontDelivery::Faces(vec![Face { format: FontFormat::Ttf, bytes, style, href: None }]);
            }
            if let FontDelivery::Faces(faces) = &mut f.delivery {
                let slug = slugify(&f.family);
                for face in faces.iter_mut() {
                    face.href = Some(format!("fonts/{}.{}", face_stem(&slug, face.style), face.format.ext()));
                }
            }
        }
    }

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
        // The renderer verified with its OWN medium's tooling (CSS: browser-grade parse + dangling
        // var() check; Typst: `typst compile`) — so the message stays generic across targets.
        println!("verified: {} {} file(s) — no problems reported", renderer.name(), outputs.len());
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
    // Linked external font files (delivery: Link) are emitted alongside the CSS and tracked too.
    let linked: Vec<(&str, &[u8])> = [&cfg.body, &cfg.heading, &cfg.code]
        .into_iter()
        .filter_map(|f| match &f.delivery {
            FontDelivery::Faces(faces) => Some(faces),
            _ => None,
        })
        .flatten()
        .filter_map(|face| face.href.as_deref().map(|h| (h, face.bytes.as_slice())))
        .collect();
    let fresh: HashSet<&str> =
        outputs.iter().map(|o| o.path.as_str()).chain(linked.iter().map(|(h, _)| *h)).collect();

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
    // Linked font files (deduped: body == heading shares one href) — written beside the CSS and
    // tracked in the manifest so a later build cleans them when they go stale, like any output.
    let mut written = HashSet::new();
    for (href, bytes) in &linked {
        if !written.insert(*href) {
            continue;
        }
        let path = b.out.join(href);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
        manifest.files.push(FileStamp { path: (*href).to_string(), checksum: checksum(bytes) });
    }
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    if !cleaned.is_empty() {
        println!("cleaned {} stale file(s): {}", cleaned.len(), cleaned.join(", "));
    }
    if !kept.is_empty() {
        println!("kept {} file(s) edited since generated (not removed): {}", kept.len(), kept.join(", "));
    }
    println!(
        "wrote {} file(s) to {} — target: {}, scale: {}, body: {}, heading: {}, code: {}, theme: {}",
        outputs.len(),
        b.out.display(),
        renderer.name(),
        cfg.scale.id(),
        cfg.body.family,
        cfg.heading.family,
        cfg.code.family,
        cfg.theme.id(),
    );
    if b.target == "typst" && !linked.is_empty() {
        println!("  compile with: typst compile --font-path {}/fonts {}/cascade.typ", b.out.display(), b.out.display());
    }
    Ok(())
}

// ── dist ───────────────────────────────────────────────────────────────────────────────────
// The release step: build the committed distribution for every renderer to dist/<format>, VERIFIED,
// then stamp the spec's VERSION and drop the build manifest (an internal clean aid, not a shipped
// artifact). One command, no shelling out, version straight from the crate. Each target's build is
// itself self-contained — the typst build writes its fonts (dist/typst/fonts), so there's nothing to
// copy in afterward.
fn dist(d: Dist) -> Res {
    for (target, sub) in [("css", "css"), ("typst", "typst")] {
        let out = d.out.join(sub);
        build(Build {
            out: out.clone(),
            target: target.into(),
            config: None,
            scale: None,
            body: None,
            heading: None,
            code: None,
            font_path: vec![],
            link_fonts: false,
            theme: None,
            verify: true,
        })?;
        let _ = std::fs::remove_file(out.join(MANIFEST)); // shipped artifact: no internal manifest
        std::fs::write(out.join("VERSION"), format!("cascade {}\n", cascade::VERSION))?;
    }
    println!("built distribution {} (cascade {})", d.out.display(), cascade::VERSION);
    Ok(())
}

/// Overlay flags onto the file onto the compiled defaults; resolve every name against its closed
/// set (an unknown value lists what IS available, rather than a silent fallback).
fn resolve(
    file: FileConfig,
    scale: Option<String>,
    body: Option<String>,
    heading: Option<String>,
    code: Option<String>,
    theme: Option<String>,
    external: &[ResolvedFont],
) -> Result<Config, String> {
    let mut cfg = Config::default();
    for s in file.scale.iter().chain(scale.iter()) {
        cfg.scale = parse_scale(s)?;
    }
    for f in file.body.iter().chain(body.iter()) {
        cfg.body = resolve_font(f, external)?;
    }
    for f in file.heading.iter().chain(heading.iter()) {
        cfg.heading = resolve_font(f, external)?;
    }
    for f in file.code.iter().chain(code.iter()) {
        cfg.code = resolve_font(f, external)?;
    }
    for t in file.theme.iter().chain(theme.iter()) {
        cfg.theme = parse_theme(t)?;
    }
    Ok(cfg)
}

/// Load external fonts under the given paths (files, or directories of them), grouping a family's
/// weight/style files into ONE font: metrics from the Regular face, delivery from the WHOLE set.
/// Inputs mix freely — font files (`.ttf`/`.otf`) are read and grouped by their family name; a
/// measured `.ron` supplies tuned metrics + category for the family of the same name (else the family
/// is measured on the fly, category defaulting to sans). A `.ron` with no font files is metrics-only.
fn load_external(paths: &[PathBuf]) -> Result<Vec<ResolvedFont>, String> {
    let is_ron = |q: &Path| q.extension().and_then(|x| x.to_str()) == Some("ron");
    let font_fmt = |q: &Path| q.extension().and_then(|x| x.to_str()).and_then(FontFormat::from_ext);

    // Gather every entry across the paths.
    let mut all: Vec<PathBuf> = Vec::new();
    for p in paths {
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(p)
                .map_err(|e| format!("read {}: {e}", p.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .collect();
            v.sort();
            all.extend(v);
        } else {
            all.push(p.clone());
        }
    }

    // Group font files by family name (name id 16 → Bold/Italic land under the family); load RONs.
    // Per family: (display name, the collected faces as (style, format, bytes)).
    type Group = (String, Vec<(FaceStyle, FontFormat, Vec<u8>)>);
    let mut groups: std::collections::BTreeMap<String, Group> = std::collections::BTreeMap::new();
    let mut rons: std::collections::BTreeMap<String, ResolvedFont> = std::collections::BTreeMap::new();
    for q in &all {
        if is_ron(q) {
            let text = std::fs::read_to_string(q).map_err(|e| format!("read {}: {e}", q.display()))?;
            let rf = cascade::measure::load_ron(&text).map_err(|e| format!("{}: {e}", q.display()))?;
            rons.insert(rf.family.to_lowercase(), rf);
        } else if let Some(format) = font_fmt(q) {
            let bytes = std::fs::read(q).map_err(|e| format!("read {}: {e}", q.display()))?;
            let info = cascade::measure::face_info(&bytes);
            let family = info
                .family
                .or_else(|| q.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .ok_or_else(|| format!("{}: could not determine a family name", q.display()))?;
            groups
                .entry(family.to_lowercase())
                .or_insert_with(|| (family, Vec::new()))
                .1
                .push((info.style, format, bytes));
        }
    }

    let mut fonts = Vec::new();
    // Each font-file family → one ResolvedFont: metrics from a matching RON (kept) or the measured
    // Regular; delivery = every face in the set (one @font-face each, correctly weighted).
    for (key, (family, files)) in groups {
        let reg = files.iter().min_by_key(|(style, _, _)| regular_score(*style)).unwrap();
        let mut rf = match rons.remove(&key) {
            Some(rf) => rf, // tuned metrics + category from the RON
            None => cascade::measure::resolve_face(&reg.2, Some(&family), Category::Sans)
                .map_err(|e| format!("{family}: {e}"))?,
        };
        rf.delivery = FontDelivery::Faces(
            files.into_iter().map(|(style, format, bytes)| Face { format, bytes, style, href: None }).collect(),
        );
        fonts.push(rf);
    }
    // RONs with no font files → metrics only (System delivery, relies on the reader having the face).
    fonts.extend(rons.into_values());
    Ok(fonts)
}

/// How close a face is to the "Regular" used for a family's metrics: upright and nearest weight 400.
fn regular_score(s: FaceStyle) -> u32 {
    let (lo, hi) = s.weight;
    (s.italic as u32) * 100_000 + (i32::from(400u16.clamp(lo, hi)) - 400).unsigned_abs()
}

/// Resolve a font family name to a `ResolvedFont`: the bundled catalog first, then the provisioned
/// externals — both selected the SAME way, by family name. An unknown name lists what IS available.
fn resolve_font(name: &str, external: &[ResolvedFont]) -> Result<ResolvedFont, String> {
    if let Some(f) = Font::from_family(name) {
        return Ok(f.into());
    }
    if let Some(f) = external.iter().find(|f| f.family.eq_ignore_ascii_case(name)) {
        return Ok(f.clone());
    }
    let bundled = Font::ALL.iter().map(|f| f.family()).collect::<Vec<_>>().join(", ");
    let ext: Vec<&str> = external.iter().map(|f| f.family.as_str()).collect();
    let ext = if ext.is_empty() { "none provisioned (use --font-path)".to_string() } else { ext.join(", ") };
    Err(format!("unknown font '{name}'. bundled: {bundled}; external: {ext}"))
}

fn parse_scale(s: &str) -> Result<ScalePreset, String> {
    ScalePreset::from_id(s).ok_or_else(|| {
        let all = ScalePreset::ALL.iter().map(|p| p.id()).collect::<Vec<_>>().join(", ");
        format!("unknown scale '{s}'. available: {all}")
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
        "typst" => Ok(Box::new(cascade_typst::Typst)),
        other => Err(format!("unknown target '{other}'. available: css, typst")),
    }
}

// ── measure ────────────────────────────────────────────────────────────────────────────────
// Thin IO wrapper: the measurement + RON format are STANDARDIZED in the spec (`cascade::measure`).
// Accepts a single font OR a directory (measures every .ttf/.otf in it — re-measuring the whole set
// of shipped fonts is one command, no task-runner recipe). Everything font-specific is the spec's.
fn measure(m: Measure) -> Res {
    let fallback_cat = Category::from_str(&m.category).ok_or_else(|| {
        format!("unknown category '{}'. available: serif, sans, mono", m.category)
    })?;
    if m.font.is_dir() {
        // TrueType and OpenType/CFF are read identically (ttf-parser auto-detects); accept either,
        // case-insensitively.
        let is_font = |p: &Path| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "ttf" | "otf"))
        };
        let mut fonts: Vec<PathBuf> = std::fs::read_dir(&m.font)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| is_font(p))
            .collect();
        fonts.sort();
        if fonts.is_empty() {
            return Err(format!("no .ttf/.otf fonts in {}", m.font.display()).into());
        }
        for font in &fonts {
            measure_one(font, None, None, fallback_cat)?;
        }
    } else {
        measure_one(&m.font, m.name.clone(), m.out.clone(), fallback_cat)?;
    }
    Ok(())
}

/// Measure ONE font → its `fonts/<name>.ron`. Re-measuring an existing font regenerates only the
/// MEASURED block and PRESERVES its category + tuned profile from the current RON (so a re-measure is
/// idempotent and needs no flags). A new font takes `fallback_cat` and a category-seeded profile.
fn measure_one(
    font: &Path,
    name_override: Option<String>,
    out_override: Option<PathBuf>,
    fallback_cat: Category,
) -> Res {
    let data = std::fs::read(font).map_err(|e| format!("read {}: {e}", font.display()))?;
    let measured = cascade::measure::measure_face(&data)?;

    let stem = font.file_stem().map(|s| s.to_string_lossy().into_owned());
    // The RON path: prefer an explicit --name, then the source FILENAME (maintainer-controlled, so a
    // re-measure maps deterministically), then the font's family. Sanitised to a valid stem — a raw
    // family may carry punctuation (Jost ships `"Jost*"`) or a variable-font suffix (`"Inter Variable"`).
    let slug_from = name_override
        .clone()
        .or_else(|| stem.clone())
        .or_else(|| measured.family.clone())
        .ok_or("could not determine a font name; pass --name")?;
    let out =
        out_override.unwrap_or_else(|| PathBuf::from(format!("cascade/fonts/{}.ron", slugify(&slug_from))));

    // Re-measure keeps the authored name + category + tuned profile from the current RON; a new font
    // uses an explicit --name (or the family/filename) and the fallback category with a seeded profile.
    let existing = std::fs::read_to_string(&out).ok();
    let name = name_override
        .or_else(|| existing.as_deref().and_then(cascade::measure::extract_name))
        .or_else(|| measured.family.clone())
        .or(stem)
        .ok_or("could not determine a font name; pass --name")?;
    let cat = existing
        .as_deref()
        .and_then(cascade::measure::extract_category)
        .and_then(|c| Category::from_str(&c))
        .unwrap_or(fallback_cat);
    let existing_profile = existing.as_deref().and_then(cascade::measure::extract_profile);
    let profile_kept = existing_profile.is_some();
    let profile = existing_profile.unwrap_or_else(|| cascade::measure::default_profile(cat));

    let ron = cascade::measure::font_ron(&name, cat, &profile, &measured);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, &ron)?;
    println!(
        "measured {} → {} (x-height {:.3}, cap {:.3}, avg-advance {:.4}, upem {}){}",
        name,
        out.display(),
        measured.x_height,
        measured.cap_height,
        measured.avg_advance,
        measured.units_per_em,
        if profile_kept { " — kept existing profile" } else { " — seeded profile from category" },
    );
    if out.starts_with("cascade/fonts") || out.components().any(|c| c.as_os_str() == "fonts") {
        println!("  the spec picks this up on the next build (drop-in + recompile).");
    }
    Ok(())
}

/// A unique file stem for a face within a family: `<slug>-<weight>[i]` (`source-serif-700`,
/// `source-serif-400i`, or `source-serif-100-900` for a variable range).
fn face_stem(slug: &str, style: FaceStyle) -> String {
    let (lo, hi) = style.weight;
    let w = if lo == hi { lo.to_string() } else { format!("{lo}-{hi}") };
    format!("{slug}-{w}{}", if style.italic { "i" } else { "" })
}

/// A font name → a filesystem-safe RON stem: lowercased, spaces to hyphens, punctuation dropped
/// (so `"Jost*"` → `jost`, `"Inter"` → `inter`).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

// ── add ────────────────────────────────────────────────────────────────────────────────────
// Fetch a family from the google/fonts repo — its font files + OFL license, plus a measured RON
// whose CATEGORY comes from Google's METADATA.pb (no guessing) and whose name is canonical. The repo
// ships TTFs (measurable, unlike the CDN's woff2). Standalone: writes to disk; `build --font-path
// <out>/<slug>` then delivers the whole family.
fn add(a: Add) -> Res {
    let slug: String = a.family.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    // The repo groups families under a license dir; try each.
    let (base, meta) = ["ofl", "apache", "ufl"]
        .iter()
        .find_map(|lic| {
            let base = format!("https://raw.githubusercontent.com/google/fonts/{}/{}/{}", a.git_ref, lic, slug);
            http_text(&format!("{base}/METADATA.pb")).ok().map(|m| (base, m))
        })
        .ok_or_else(|| format!("family '{}' not found in google/fonts (looked up slug '{slug}')", a.family))?;

    let field = |key: &str| {
        meta.lines().find_map(|l| l.trim().strip_prefix(key).map(|v| v.trim().trim_matches('"').to_string()))
    };
    let name = field("name:").unwrap_or_else(|| a.family.clone());
    let category = match field("category:").as_deref() {
        Some("SERIF") => "serif",
        Some("MONOSPACE") => "mono",
        _ => "sans", // SANS_SERIF / DISPLAY / HANDWRITING → sans
    };
    let cat = Category::from_str(category).unwrap();
    let files: Vec<String> = meta
        .lines()
        .filter_map(|l| l.trim().strip_prefix("filename:").map(|v| v.trim().trim_matches('"').to_string()))
        .collect();
    if files.is_empty() {
        return Err("METADATA.pb lists no font files".into());
    }

    let dir = a.out.join(&slug);
    std::fs::create_dir_all(&dir)?;
    if let Ok(ofl) = http_text(&format!("{base}/OFL.txt")) {
        std::fs::write(dir.join("OFL.txt"), ofl)?;
    }
    // Download every file; keep the "regular-most" for the measured RON (a variable file covers all).
    let mut regular: Option<Vec<u8>> = None;
    for f in &files {
        let bytes = http_bytes(&format!("{base}/{}", urlencode(f)))?;
        std::fs::write(dir.join(f), &bytes)?;
        if regular.is_none() && (f.contains('[') || f.to_lowercase().contains("regular")) {
            regular = Some(bytes);
        }
    }
    let regular =
        regular.or_else(|| std::fs::read(dir.join(&files[0])).ok()).ok_or("no face to measure")?;

    // Measure the regular → a RON carrying Google's category + canonical name, so build gets both right.
    let measured = cascade::measure::measure_face(&regular)?;
    let ron = cascade::measure::font_ron(&name, cat, &cascade::measure::default_profile(cat), &measured);
    std::fs::write(dir.join(format!("{slug}.ron")), ron)?;

    println!("added {name} ({category}) → {} — {} file(s) + OFL + {slug}.ron", dir.display(), files.len());
    println!("  build with: cascade build --font-path {} --body {name:?}", dir.display());
    Ok(())
}

/// GET a URL as text (METADATA.pb, OFL.txt).
fn http_text(url: &str) -> Result<String, String> {
    ureq::get(url).call().map_err(|e| e.to_string())?.into_string().map_err(|e| e.to_string())
}

/// GET a URL as bytes (a font file).
fn http_bytes(url: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut buf = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|e| e.to_string())?
        .into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

/// Percent-encode the URL-unsafe characters google/fonts filenames carry (`[ ] , space`).
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '[' => "%5B".to_string(),
            ']' => "%5D".to_string(),
            ',' => "%2C".to_string(),
            ' ' => "%20".to_string(),
            c => c.to_string(),
        })
        .collect()
}

// ── list ───────────────────────────────────────────────────────────────────────────────────
fn list() -> Res {
    let d = Config::default();
    println!("scales (--scale):");
    for p in ScalePreset::ALL {
        let tag = if p == d.scale { "  (default)" } else { "" };
        println!("  {}{}", p.id(), tag);
    }
    println!("\nfonts (--body / --heading / --code):");
    for f in Font::ALL {
        let mut tags = Vec::new();
        if d.body.family == f.family() {
            tags.push("body-default");
        }
        if d.heading.family == f.family() {
            tags.push("heading-default");
        }
        if d.code.family == f.family() {
            tags.push("code-default");
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
