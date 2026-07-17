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
| `ibmplexmono.ron` | IBM Plex Mono | mono (code default) |

`default_fonts` in `tokens.ron` binds each font-role (`body` / `heading` / `code`) to one of these, so
code renders the bundled IBM Plex Mono with its own measured optical, exactly as body renders Inter. A
multi-word family compiles to a PascalCase `Font` variant (`Font::IBMPlexMono`) and slugifies to a single
CSS token (`.bundle-ibmplexmono`).

## Sources carry with the spec

`sources/` holds the actual font files (`inter.ttf`, `lora.ttf`, `ibmplexmono.ttf`) and their SIL OFL
licenses (`*.OFL.txt`) — vendored so the bundle is fully self-contained and re-measurable **offline, with
no external tooling**. All three are OFL 1.1 (redistributable). Regenerate every bundled RON from them
in one command:

    cargo run -p cascade-cli -- measure cascade/fonts/sources     # → inter.ron, lora.ron, ibmplexmono.ron

Idempotent, and preserves each font's tuned `profile`. `sources/` holds the **variable** fonts —
`measure_face` samples them: it pins the optical axis to its text end and averages the weight axis
(a bold-skewed Gaussian prior over the wght range, `m* = pᵀM`), since x-height/cap are weight-
invariant and only the advance drifts. So the measured metric is a robust family value, not one
arbitrary instance. (`build.rs` scans only `*.ron` at this folder's top level, so `sources/`,
`faces/`, this README, and the `.OFL.txt` files are ignored by the spec compile.)

## `faces/` — static delivery faces (for Typst)

`faces/` holds **static** instances (Regular/SemiBold/Bold + italics) for renderers that can't use a
variable font — Typst 0.14 renders a `weight: 700` request from a variable file at *regular*. The CSS
path uses the variable sources directly (`@font-face`); the Typst dist ships these static faces
(`cascade build --target typst` / `dist` copies them to `dist/typst/fonts`, compiled with
`--font-path fonts`). They are instanced from the variable sources (opsz pinned to text), e.g.:

    fonttools varLib.instancer sources/lora.ttf wght=700 -o faces/Lora-Bold.ttf

Metrics come from the variable measurement above (weight-invariant), so the statics are delivery
only — never separately measured.

## Add / update a bundled font

Drop the source into `sources/` (`.ttf` or `.otf` — read identically), measure it, and recompile;
the `Font` enum picks up the new RON:

    cargo run -p cascade-cli -- measure cascade/fonts/sources/<name>.ttf     # → cascade/fonts/<name>.ron

## Not here: external fonts

Web and system fonts are **the client's domain**, not bundled. They're measured at CLI *runtime*
into the same shape (a `ResolvedFont`) and loaded on demand — they never enter this folder or the
compiled `Font` enum. The line: **bundled = carried in the spec; external = sourced client-side.**
