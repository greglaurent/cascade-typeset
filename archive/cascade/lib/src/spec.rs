//! The cascade typographic SPEC — the engine, decoded from `tokens.ron` (RON + serde).
//! The deep definitions: scale math, optical model, theme, rhythm, and the generic font
//! families. The consumer-facing "supported + packaged" surface lives in [`crate::manifest`]
//! (cascade.ron). Numbers live here; the FORMULAS live in each renderer's templates.
//!
//! `deny_unknown_fields` on every record makes a mistyped key a decode error rather than a
//! silent drop — the first correctness gate (shape). Domain truths live in [`crate::validate`].
use serde::Deserialize;
use std::path::Path;

/// Load and decode `tokens.ron` into the typed spec. A type mismatch, missing field, or
/// unknown key fails here with a located error.
pub fn load(path: &Path) -> Result<Spec, Box<dyn std::error::Error>> {
    Ok(ron::from_str(&std::fs::read_to_string(path)?)?)
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub scale: Scale,
    pub optical: Optical,
    pub theme: Theme,
    pub rhythm: Rhythm,
    pub generics: Generics,
}

// ── scale ─────────────────────────────────────────────────────────────────────
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Scale {
    pub base: MediaStr,
    pub steps: Steps,
    pub default: String,
    pub presets: Vec<Preset>,
}

/// A target-specific string pair: `print` (Typst/PDF) vs `web` (CSS).
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MediaStr {
    pub print: String,
    pub web: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub min: i32,
    pub max: i32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub name: String,
    pub ratio: f64,
    pub n: u32,
}

// ── optical ───────────────────────────────────────────────────────────────────
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Optical {
    pub word_space_k: f64,
    pub tracking_clamp: f64,
    pub leading_clamp: MinMax,
    pub measure: u32,
    pub size_min: MediaStr,
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

/// A named optical profile (tracking/leading/word-space knobs).
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub optical_size: String,
    pub x_height: f64,
    pub k_tracking: f64,
    pub leading_base: f64,
    pub word_space: f64,
}

// ── theme ─────────────────────────────────────────────────────────────────────
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub light: Vec<Color>,
    pub dark: Vec<Color>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Color {
    pub name: String,
    pub hex: String,
}

// ── rhythm ────────────────────────────────────────────────────────────────────
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Rhythm {
    pub unit: Unit,
    pub multipliers: Vec<Multiplier>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub print: String,
    pub web: f64,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Multiplier {
    pub name: String,
    pub value: f64,
}

// ── generic families ──────────────────────────────────────────────────────────
/// The built-in generic families + their optical-profile pairings.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Generics {
    pub default: String,
    pub stacks: Vec<Stack>,
    pub bundles: Vec<Bundle>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub name: String,
    pub typst: String,
    pub css: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub name: String,
    pub stack: String,
    pub profile: String,
}
