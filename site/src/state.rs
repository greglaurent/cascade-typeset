//! Shared application state.

use std::sync::Arc;

use crate::assets::Assets;

#[derive(Clone)]
pub struct AppState {
    /// Hashed asset URLs from Vite's manifest (the frontend bundle: datastar + Pico + site CSS).
    pub assets: Assets,
    /// The cascade stylesheet, rendered once in-memory via the cascade-css API. Deterministic for a
    /// given Config, so there's no need to re-render per request; shared cheaply via `Arc`.
    pub stylesheet: Arc<String>,
}
