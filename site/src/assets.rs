//! Resolve content-hashed asset URLs from Vite's build manifest, so templates reference the
//! current hashed filenames (automatic cache-busting). Unlike wine-app this is SOFT: if the
//! frontend hasn't been built yet, the server still runs and serves the page (styled by the
//! dogfooded cascade CSS), just without the datastar bundle until `just build-web`.

use std::collections::HashMap;

use serde::Deserialize;

const MANIFEST: &str = "web/dist/.vite/manifest.json";
const ENTRY: &str = "src/main.ts";

#[derive(Clone, Default)]
pub struct Assets {
    pub site_js: String,
    pub site_css: String,
}

#[derive(Deserialize)]
struct ManifestEntry {
    file: String,
    #[serde(default)]
    css: Vec<String>,
}

impl Assets {
    /// Resolve at startup; on failure log a hint and return empty URLs (the page still serves).
    pub fn load() -> Self {
        match Self::resolve() {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(
                    "frontend not built ({e}) -- run `just build-web`; serving without datastar until then"
                );
                Assets::default()
            }
        }
    }

    /// The current hashed assets. In DEBUG re-reads the manifest each call, so a `pnpm build` is
    /// picked up without a server restart; in RELEASE returns the values cached at startup.
    pub fn current(&self) -> Self {
        #[cfg(debug_assertions)]
        if let Ok(fresh) = Self::resolve() {
            return fresh;
        }
        self.clone()
    }

    fn resolve() -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(MANIFEST).map_err(|e| anyhow::anyhow!("{MANIFEST}: {e}"))?;
        let map: HashMap<String, ManifestEntry> = serde_json::from_str(&raw)?;
        let entry = map
            .get(ENTRY)
            .ok_or_else(|| anyhow::anyhow!("`{ENTRY}` missing from Vite manifest"))?;
        Ok(Assets {
            site_js: format!("/{}", entry.file),
            site_css: entry.css.first().map(|c| format!("/{c}")).unwrap_or_default(),
        })
    }
}
