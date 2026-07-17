//! cascade-site -- Axum server for the public cascade site. Serves the specimen page and the
//! cascade stylesheet, rendered IN-MEMORY via the cascade-css API; the interactive controls are
//! datastar signals, resolved entirely in the browser. The frontend bundle (datastar + Pico) comes
//! from the Vite build.

mod assets;
mod css;
mod state;
mod web;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use assets::Assets;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());

    let state = AppState {
        assets: Assets::load(),
        stylesheet: Arc::new(css::stylesheet()),
    };

    let app = Router::new()
        .route("/", get(web::index))
        .route("/sample", get(web::sample))
        .route("/health", get(|| async { "ok" }))
        // The cascade stylesheet, rendered in-memory via the cascade-css API (single source: the renderer).
        .route("/css/cascade.css", get(web::cascade_css))
        // Serve the bundled font sources directly from the spec crate (no duplication) so the
        // specimen can @font-face the real Inter/Lora instead of falling back to system faces.
        .nest_service("/fonts", ServeDir::new("../cascade/fonts/sources"))
        // Vite build output (hashed JS/CSS, favicon, brand assets).
        .fallback_service(ServeDir::new("web/dist"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = bind.parse()?;
    tracing::info!("cascade-site listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
