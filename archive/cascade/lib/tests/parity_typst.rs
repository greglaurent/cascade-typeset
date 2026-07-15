//! Byte-parity gate for the Typst renderer: each template rendered from tokens.ron +
//! cascade.ron must be byte-identical to the file the legacy `just gen` produced. This
//! is the M2 acceptance criterion — one assert per template as they land.
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

/// Read a repo-root-relative generated file (cascade/lib/../.. == repo root).
fn current(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join("../..").join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn scale_typ_is_byte_identical() {
    assert_eq!(
        cascade_lib::render::scale_typ(&spec()),
        current("cascade-typst/scale.typ"),
        "rendered scale.typ differs from cascade-typst/scale.typ",
    );
}

#[test]
fn rhythm_typ_is_byte_identical() {
    assert_eq!(
        cascade_lib::render::rhythm_typ(&spec()),
        current("cascade-typst/rhythm.typ"),
        "rendered rhythm.typ differs from cascade-typst/rhythm.typ",
    );
}

#[test]
fn theme_typ_is_byte_identical() {
    assert_eq!(
        cascade_lib::render::theme_typ(&spec()),
        current("cascade-typst/theme.typ"),
        "rendered theme.typ differs from cascade-typst/theme.typ",
    );
}

#[test]
fn font_typ_is_byte_identical() {
    assert_eq!(
        cascade_lib::render::font_typ(&spec(), &manifest()),
        current("cascade-typst/font.typ"),
        "rendered font.typ differs from cascade-typst/font.typ",
    );
}
