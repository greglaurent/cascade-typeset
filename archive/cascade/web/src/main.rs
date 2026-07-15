// cascade-web — dev server that serves the examples (viewer + POST /compile).
//
// Scaffold only. The real server lands in M5: port site/serve.ts (static file
// serving from the repo root, `/` → site/index.html, the typst-compile endpoint
// with its option allowlist), at which point the Deno toolchain goes away and the
// HTTP layer (tiny-http vs axum) is chosen. Kept as a compiling stub so the
// workspace builds end-to-end from M0.
fn main() {
    eprintln!("cascade-web: scaffold — real server lands in M5 (see site/serve.ts)");
}
