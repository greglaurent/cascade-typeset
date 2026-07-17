import { defineConfig } from "vite";

// Bundles the site entry (src/main.ts -> Pico + site CSS + vendored datastar), content-hashed,
// with a manifest the Rust server reads to resolve asset URLs. No wasm/PWA/worker: unlike
// wine-app this site is stateless, so the frontend is just styling + datastar.
export default defineConfig({
  build: {
    target: "esnext",
    outDir: "dist",
    emptyOutDir: true,
    manifest: true,
    rollupOptions: {
      input: { main: "src/main.ts" },
    },
  },
});
