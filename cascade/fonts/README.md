# cascade — bundled fonts

The fonts that ship **with the spec**. Each file is one `<name>.ron`: a font's measured optical
metrics (`measured`) plus its tuned optical profile (`profile`). `build.rs` compiles every RON in
this folder into the `Font` enum, so a bundled font is a first-class, valid-by-construction member
of the catalog — the defaults and the `--body`/`--heading` choices are drawn from here.

Currently bundled:

| RON | family | category |
|---|---|---|
| `inter.ron` | Inter | sans (body default) |
| `lora.ron` | Lora | serif (heading default) |

## Sources carry with the spec

`sources/` holds the actual font files (`inter.ttf`, `lora.ttf`) and their SIL OFL licenses
(`*.OFL.txt`) — vendored so the bundle is fully self-contained and re-measurable **offline, with no
external tooling**. Both fonts are OFL 1.1 (redistributable). Regenerate every bundled RON from them
in one command:

    cargo run -p cascade-cli -- measure cascade/fonts/sources     # → inter.ron, lora.ron

This is idempotent — byte-identical to what's committed — and preserves each font's tuned `profile`.
(`build.rs` scans only `*.ron` at this folder's top level, so `sources/`, this README, and the
`.OFL.txt` files are ignored by the spec compile.)

## Add / update a bundled font

Drop the source into `sources/` (`.ttf` or `.otf` — read identically), measure it, and recompile;
the `Font` enum picks up the new RON:

    cargo run -p cascade-cli -- measure cascade/fonts/sources/<name>.ttf     # → cascade/fonts/<name>.ron

## Not here: external fonts

Web and system fonts are **the client's domain**, not bundled. They're measured at CLI *runtime*
into the same shape (a `ResolvedFont`) and loaded on demand — they never enter this folder or the
compiled `Font` enum. The line: **bundled = carried in the spec; external = sourced client-side.**
