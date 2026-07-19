//! Shared application state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::assets::Assets;

#[derive(Clone)]
pub struct AppState {
    /// Hashed asset URLs from Vite's manifest (the frontend bundle: datastar + Pico + site CSS).
    pub assets: Assets,
    /// The cascade stylesheet, rendered once in-memory via the cascade-css API. Deterministic for a
    /// given Config, so there's no need to re-render per request; shared cheaply via `Arc`.
    pub stylesheet: Arc<String>,
    /// Compiled specimen PDFs, keyed by option-signature. A `typst compile` is ~hundreds of ms and
    /// the output is deterministic per Config, so we cache the bytes (the reactive UI re-requests on
    /// every option change). Shared cheaply via `Arc`.
    pub pdf_cache: Arc<Mutex<HashMap<String, Arc<Vec<u8>>>>>,
}
