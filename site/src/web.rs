//! Page + asset handlers.

use askama::Template;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::state::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    site_js: String,
    site_css: String,
}

pub async fn index(State(st): State<AppState>) -> Result<Html<String>, StatusCode> {
    let a = st.assets.current();
    IndexTemplate {
        title: "Cascade -- type-driven typography".into(),
        site_js: a.site_js,
        site_css: a.site_css,
    }
    .render()
    .map(Html)
    .map_err(|e| {
        tracing::error!("render index: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Template)]
#[template(path = "sample.html")]
struct SampleTemplate;

/// The specimen the chrome loads in its iframe (isolated from Pico, styled by cascade).
pub async fn sample() -> Result<Html<String>, StatusCode> {
    SampleTemplate
        .render()
        .map(Html)
        .map_err(|e| {
            tracing::error!("render sample: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// The cascade stylesheet, rendered in-memory via the cascade-css API.
pub async fn cascade_css(State(st): State<AppState>) -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        (*st.stylesheet).clone(),
    )
        .into_response()
}
