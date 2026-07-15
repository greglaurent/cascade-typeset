//! Byte-parity gate for the CSS renderer: each file rendered from tokens.ron +
//! cascade.ron must be byte-identical to what the legacy `just gen` produced.
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn spec() -> cascade_lib::spec::Spec {
    cascade_lib::spec::load(&manifest_dir().join("tokens.ron")).expect("load tokens.ron")
}

fn manifest() -> cascade_lib::manifest::Manifest {
    cascade_lib::manifest::load(&manifest_dir().join("cascade.ron")).expect("load cascade.ron")
}

fn current(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join("../..").join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn scale_css_is_byte_identical() {
    assert_eq!(cascade_lib::render::scale_css(&spec()), current("cascade-css/scale.css"));
}

#[test]
fn rhythm_css_is_byte_identical() {
    assert_eq!(cascade_lib::render::rhythm_css(&spec()), current("cascade-css/rhythm.css"));
}

#[test]
fn theme_css_is_byte_identical() {
    assert_eq!(cascade_lib::render::theme_css(&spec()), current("cascade-css/theme.css"));
}

#[test]
fn font_css_is_byte_identical() {
    assert_eq!(cascade_lib::render::font_css(&spec()), current("cascade-css/font.css"));
}

#[test]
fn font_preset_css_all_byte_identical() {
    for f in &manifest().fonts {
        assert_eq!(
            cascade_lib::render::font_preset_css(f),
            current(&format!("cascade-css/fonts/{}.css", f.name)),
            "fonts/{}.css differs",
            f.name,
        );
    }
}
