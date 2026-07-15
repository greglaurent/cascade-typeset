// cascade-lib — the generator CLI (wired to `just gen`).
// Loads tokens.ron + cascade.ron, validates the typographic invariants, and writes
// every generated file into cascade-css/ and cascade-typst/ at the repo root.
use std::path::PathBuf;

fn main() {
    let lib_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // cascade/lib
    let repo_root = lib_dir.join("../.."); // tokens/outputs live at the repo root

    let spec = cascade_lib::spec::load(&lib_dir.join("tokens.ron")).unwrap_or_else(die);
    let manifest = cascade_lib::manifest::load(&lib_dir.join("cascade.ron")).unwrap_or_else(die);

    // Domain gate: never write from an invalid spec.
    if let Err(problems) = cascade_lib::validate::validate(&spec, &manifest) {
        eprintln!("cascade-lib: spec/manifest invalid, not writing:");
        for p in &problems {
            eprintln!("  - {p}");
        }
        std::process::exit(1);
    }

    for (rel, body) in cascade_lib::render::files(&spec, &manifest) {
        if let Err(e) = std::fs::write(repo_root.join(&rel), body) {
            eprintln!("cascade-lib: write {rel}: {e}");
            std::process::exit(1);
        }
        println!("wrote {rel}");
    }
}

fn die<T>(err: Box<dyn std::error::Error>) -> T {
    eprintln!("cascade-lib: {err}");
    std::process::exit(1);
}
