//! The spec, generated at build time by `build.rs` from tokens.ron + theme.ron + themes/ + fonts/,
//! and included here. Every closed set in the spec becomes an enum whose variants are the defined
//! values, with data accessors -- so consumers are handed values valid by construction. The
//! generated types are plain Rust with ZERO runtime deps (the spec *is* the type system).
//!
//! `build.rs` reads the RON from CARGO_MANIFEST_DIR and writes the generated code to OUT_DIR, both
//! of which cargo sets correctly for EVERY build -- so this works whether `cascade` is built
//! directly or consumed as a path/git/registry dependency in any workspace.
include!(concat!(env!("OUT_DIR"), "/spec.rs"));
