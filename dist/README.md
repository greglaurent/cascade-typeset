# cascade distribution

The **shipped, pre-built** cascade output -- the artifact a consumer includes, grouped by renderer
format. These files are committed and are cascade's release deliverable; they are NOT generated at
a consumer's build time. Regenerate with `just dist` (compiles once, here) and commit the result.

| Directory | Format | Include |
|---|---|---|
| [`css/`](css/) | CSS | `css/cascade.css` (it `@import`s the seven modules) |
| `typst/` | Typst | pending -- lands with the `cascade-typst` renderer |

`css/VERSION` records the cascade version the CSS was built from. The build uses the spec's default
config (golden-ditonic scale, Inter/Lora, paper theme); palette, light/dark, typeface, and scale
are still switchable at runtime via the classes/attributes the CSS emits. Consumers who need a
different *baked* default run the CLI themselves (`cascade build --scale ... --out ...`).
