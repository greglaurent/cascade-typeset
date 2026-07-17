# cascade distribution

The **shipped, pre-built** cascade output -- the artifact a consumer includes, grouped by renderer
format. These files are committed and are cascade's release deliverable; they are NOT generated at
a consumer's build time. Regenerate with `cargo run -p cascade-cli -- dist` (compiles once, here) and
commit the result.

| Directory | Format | Include |
|---|---|---|
| [`css/`](css/) | CSS | `css/cascade.css` (it `@import`s the seven modules) |
| [`typst/`](typst/) | Typst | `typst/cascade.typ` (`#import "cascade.typ": make`); compile with `--font-path fonts` |

`<format>/VERSION` records the cascade version each was built from. The build uses the spec's default
config (golden-ditonic scale, Inter/Lora/IBM Plex Mono, paper theme). For **CSS**, palette,
light/dark, typeface, and scale stay switchable at runtime via the classes/attributes the CSS emits.
**Typst** is print: there's no runtime, so the config is BAKED (numeric projection of the same spec
formulas), and `typst/fonts/` ships the static faces Typst needs — it can't weight a variable font,
so real Regular/SemiBold/Bold + italics are provided (`typst compile --font-path fonts your.typ`).
Consumers who need a different *baked* default run the CLI themselves (`cascade build --target typst
--scale ... --out ...`).
