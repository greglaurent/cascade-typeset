//! Typst PDF rendering — the print projection of the specimen, compiled on demand.
//!
//! The site is a Rust consumer of BOTH renderers: [`crate::css`] renders the CSS live, this renders
//! the PDF live. For a set of options it builds a [`Config`], has `cascade-typst` bake `cascade.typ`,
//! and shells out to the `typst` CLI to compile the shared specimen (`templates/sample.typ`) against
//! it — the same document the CSS tab shows, in print form. Results are cached by option-signature.
//!
//! Two honest projection gaps (print has no analogue): the CSS *category* aliases (`serif`/`sans`/
//! `mono`) are generic web stacks, so each maps to the bundled face of its category; and the theme
//! toggle is light/dark while print is paper — the PDF always bakes the light palette.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use cascade::renderer::{Config, Renderer};
use cascade::{Font, ScalePreset};
use cascade_typst::Typst;
use serde::Deserialize;

use crate::state::AppState;

/// The shared specimen, mirroring `templates/sample.html`. Static content — the config is baked into
/// `cascade.typ`; the notes edition is the one `--input`.
const SPECIMEN: &str = include_str!("../templates/sample.typ");

/// The bundled variable font sources (Inter / Lora / IBM Plex Mono), passed to `typst --font-path`
/// so the baked family names resolve to the real faces (same dir the site serves at `/fonts`).
const FONT_SOURCES: &str = "../cascade/fonts/sources";

/// The option set from the query string (raw UI select values; prefixes stripped on resolve).
#[derive(Deserialize)]
pub struct Opts {
    #[serde(default)]
    scale: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    heading: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    notes: String,
}

/// A UI option token → a bundled [`Font`]. The category aliases (`serif`/`sans`/`mono`) are generic
/// web stacks with no print analogue, so each maps to the bundled face of its category; explicit
/// `lora`/`inter` map to themselves. Empty → `None` (caller supplies the role's default).
fn font_of(token: &str) -> Option<Font> {
    let bare = token
        .trim_start_matches("bundle-")
        .trim_start_matches("heading-")
        .trim_start_matches("code-");
    match bare {
        "serif" | "lora" => Some(Font::Lora),
        "sans" | "inter" => Some(Font::Inter),
        "mono" => Some(Font::IBMPlexMono),
        "" => None,
        other => Font::from_family(other),
    }
}

/// Build the [`Config`] the options select. The specimen's default typeface is the serif bundle
/// (Lora), headings match the body when unset, and code defaults to the bundled IBM Plex Mono.
fn config(o: &Opts) -> Config {
    let mut cfg = Config::default();
    if let Some(s) = ScalePreset::from_id(o.scale.trim_start_matches("scale-")) {
        cfg.scale = s;
    }
    let body = font_of(&o.body).unwrap_or(Font::Lora);
    cfg.body = body.into();
    cfg.heading = font_of(&o.heading).unwrap_or(body).into();
    cfg.code = font_of(&o.code).unwrap_or(Font::IBMPlexMono).into();
    cfg
}

/// Bake `cascade.typ` for `cfg` and compile the specimen with the `typst` CLI, returning PDF bytes
/// (or the compiler's diagnostics on failure).
fn compile(cfg: &Config, sidenotes: bool) -> Result<Vec<u8>, String> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "cascade-site-pdf-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("scratch dir: {e}"))?;

    let lib = Typst
        .render(cfg)
        .into_iter()
        .find(|o| o.path == "cascade.typ")
        .ok_or("cascade-typst produced no cascade.typ")?;
    std::fs::write(dir.join("cascade.typ"), &lib.body).map_err(|e| format!("write cascade.typ: {e}"))?;
    std::fs::write(dir.join("sample.typ"), SPECIMEN).map_err(|e| format!("write sample.typ: {e}"))?;

    let out_pdf = dir.join("out.pdf");
    let fonts = std::fs::canonicalize(FONT_SOURCES).unwrap_or_else(|_| FONT_SOURCES.into());
    let result = Command::new("typst")
        .arg("compile")
        .arg("--root")
        .arg(&dir)
        .arg("--font-path")
        .arg(&fonts)
        .arg("--input")
        .arg(format!("sidenotes={sidenotes}"))
        .arg(dir.join("sample.typ"))
        .arg(&out_pdf)
        .output();

    let out = match result {
        Ok(o) if o.status.success() => std::fs::read(&out_pdf).map_err(|e| format!("read pdf: {e}")),
        Ok(o) => Err(format!("typst compile failed:\n{}", String::from_utf8_lossy(&o.stderr))),
        Err(e) => Err(format!("could not run `typst` (is it installed?): {e}")),
    };
    let _ = std::fs::remove_dir_all(&dir);
    out
}

/// `GET /sample.pdf?scale=…&body=…&heading=…&code=…&notes=…` — the print projection of the specimen.
pub async fn sample_pdf(State(st): State<AppState>, Query(o): Query<Opts>) -> Response {
    let cfg = config(&o);
    let sidenotes = o.notes == "banded";
    let sig = format!(
        "{}|{}|{}|{}|{sidenotes}",
        cfg.scale.id(),
        cfg.body.family,
        cfg.heading.family,
        cfg.code.family,
    );

    if let Some(bytes) = st.pdf_cache.lock().unwrap().get(&sig).cloned() {
        return pdf_response(bytes);
    }

    // `typst compile` blocks and shells out — keep it off the async runtime's worker.
    match tokio::task::spawn_blocking(move || compile(&cfg, sidenotes)).await {
        Ok(Ok(bytes)) => {
            let arc = Arc::new(bytes);
            st.pdf_cache.lock().unwrap().insert(sig, arc.clone());
            pdf_response(arc)
        }
        Ok(Err(e)) => {
            tracing::error!("pdf compile: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("compile task failed: {e}")).into_response(),
    }
}

fn pdf_response(bytes: Arc<Vec<u8>>) -> Response {
    ([(header::CONTENT_TYPE, "application/pdf")], (*bytes).clone()).into_response()
}
