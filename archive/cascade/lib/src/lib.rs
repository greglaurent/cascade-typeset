//! cascade-lib — the renderer. Two decoded inputs:
//!   • [`spec`]     — the typographic engine (tokens.ron): scale, optical, theme, rhythm, generics.
//!   • [`manifest`] — the build's supported + packaged surface (cascade.ron): exposed
//!     options + the specific typefaces it ships.
//! [`validate`] enforces the typographic invariants types can't express. From M2 on, the
//! inputs render through askama templates into cascade-css/ and cascade-typst/.
pub mod manifest;
pub mod render;
pub mod spec;
pub mod validate;
