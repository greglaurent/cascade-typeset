//! CSS grammar gate: every generated stylesheet must actually parse as CSS. askama
//! validates the template layer + Rust types, but treats CSS outside `{{ }}` as opaque
//! text — a malformed `calc()`/`clamp()` or a missing brace would sail through. Parsing
//! the rendered output with lightningcss closes that blind spot. (The Typst half is the
//! `typst compile` step in `just verify`.)
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use std::path::PathBuf;

fn spec() -> cascade_lib::spec::Spec {
    cascade_lib::spec::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tokens.ron")).expect("load tokens.ron")
}

fn manifest() -> cascade_lib::manifest::Manifest {
    cascade_lib::manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cascade.ron")).expect("load cascade.ron")
}

fn assert_parses(name: &str, css: &str) {
    if let Err(e) = StyleSheet::parse(css, ParserOptions::default()) {
        panic!("{name}: CSS grammar error: {e:?}");
    }
}

#[test]
fn generated_css_is_valid_grammar() {
    let spec = spec();
    assert_parses("scale.css", &cascade_lib::render::scale_css(&spec));
    assert_parses("rhythm.css", &cascade_lib::render::rhythm_css(&spec));
    assert_parses("theme.css", &cascade_lib::render::theme_css(&spec));
    assert_parses("font.css", &cascade_lib::render::font_css(&spec));
    for f in &manifest().fonts {
        assert_parses(&format!("fonts/{}.css", f.name), &cascade_lib::render::font_preset_css(f));
    }
}
