//! The shipped spec + manifest must be typographically valid. This test is the gate:
//! `cargo test` fails if tokens.ron / cascade.ron violate any invariant in
//! `cascade_lib::validate`. (Move the same call into build.rs behind a build-dependency
//! to make it fail `cargo build` outright.)
use std::path::PathBuf;

#[test]
fn shipped_spec_and_manifest_are_valid() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let spec = cascade_lib::spec::load(&dir.join("tokens.ron")).expect("load tokens.ron");
    let manifest = cascade_lib::manifest::load(&dir.join("cascade.ron")).expect("load cascade.ron");

    if let Err(problems) = cascade_lib::validate::validate(&spec, &manifest) {
        panic!("spec/manifest invalid:\n  - {}", problems.join("\n  - "));
    }
}
