# cascade-typeset

One typography system, two renderers that mirror each other module-for-module:

- **[cascade-typst/](cascade-typst/)** — the source Typst library (`@local/cascade`).
- **[cascade-css/](cascade-css/)** — the CSS port: same scale, optical profile, rhythm, and theme, driven by custom properties.

Both express the same model — a modular scale, a size-dependent optical profile
(tracking / leading / word-space), vertical rhythm, a light/dark theme, and
per-category font bundles — so a document reads the same whether it's set in
Typst or on the web. They live in one repo on purpose: the two are faithful
mirrors, and this is where they stay in sync.

## Single source of truth

Every tunable **number** in the system lives once, in **`tools/tokens.mjs`** — scale
ratios, optical coefficients, the light palette, rhythm multipliers, and the
measured font metrics. `just gen` regenerates the token-driven parts of both
renderers from it, so they're identical by construction and can't drift. The
**formulas** that consume the numbers stay in each renderer (CSS `pow()`/`log()`,
Typst `make()`); only `layout.{css,typ}` is fully hand-written (structure, not
numbers). The generated files carry a `GENERATED … do not edit` header and are
committed (dead-tongue reads them directly) — **edit `tools/tokens.mjs`, run `just gen`.**
The token file carries a JSDoc `@ts-check` schema, so a typo or wrong-typed value is
flagged in your editor (and by `just typecheck`) before it ever reaches the generator.

## Layout

| path | what |
|---|---|
| `tools/tokens.mjs` | the numbers — single source of truth |
| `tools/gen.mjs`    | generator: tokens → both renderers (`just gen`) |
| `tools/verify.mjs` | fidelity checks (`just verify`) |
| `cascade-typst/`   | Typst library: `theme` · `scale` · `font` · `rhythm` · `layout` (+ `utils`), entrypoint `lib.typ` |
| `cascade-css/`     | CSS port: `theme.css` · `scale.css` · `font.css` · `rhythm.css` · `layout.css`, `fonts/` presets, `sample.html` |
| `index.html` · `serve.ts` | the dev viewer (CSS ⇄ Typst) served by `just serve` |
| `deno.jsonc`       | Deno config (relaxed `checkJs` for the tools) |

## Dev

The maintainer toolchain runs on **Deno** — one runtime for everything. Deno ships
Node's type definitions built-in, so `gen`/`verify`/`serve` are type-checked with **no
`@types/node` and no `node_modules`. (The shipped `cascade-css/` and `cascade-typst/`
libraries need neither Deno nor Node — they're plain CSS and a Typst package.)

```
just gen       # regenerate both renderers from tokens.mjs
just verify    # fidelity: generated files in sync with tokens + Typst formulas vs the model
just typecheck # type-check the toolchain (deno check — Node types built-in)
just serve     # serve both examples at http://localhost:8175 (CSS ⇄ Typst viewer)
just pdf       # compile the Typst example → sample.pdf
```

Any Typst document can `#import "@local/cascade:0.1.0": layout, font`. That `@local`
link is provided declaratively by the nixos config (`modules/home/typst.nix`); on a
non-nix machine, symlink `cascade-typst/` into Typst's local-package namespace (see
the note atop the `justfile`).

## Font presets

Font-specific presets pin a typeface's **measured** optical metrics (x-height,
tracking, word-space) so leading and spacing are tuned per face rather than to a
generic profile. Add one to `tokens.fonts.presets` (x-height from `fonttools`'
OS/2 read) and `just gen` emits both the Typst bundle and `cascade-css/fonts/<name>.css`
— guaranteed in sync. Lora (serif), Inter (sans), and Jost (geometric sans) ship as presets.

## Sidenotes & margin notes (Tufte, opt-in)

Numbered **sidenotes** and unnumbered **margin notes**, in both renderers:

- **Typst** — one flag, two editions from the same source. `layout.make(sidenotes: true)`
  is the **margin edition**: a wide outer margin with `sidenote` (numbered) and `marginnote`
  (unnumbered) placed via the `marge` package (cascade's one external Typst dep).
  `sidenotes: false` (the **default, standards edition**) drops the same `sidenote`/`marginnote`
  calls to ordinary numbered footnotes at the foot of the page — submission-safe, no source
  changes. (`marginnote` has no unnumbered footer form, so it becomes a numbered footnote too.)
- **CSS** — load `cascade-css/sidenotes.css`, add `.sidenotes` plus a layout modifier
  to the `.cascade` box, and use the sidenote markup. Two layouts:
  - `.centered` — text stays centered at its measure; notes collapse to their markers,
    and clicking any marker opens them all inline as indented asides (clicking again
    closes them — a pure-CSS `:has()` disclosure). Collapsed, the paragraph reads intact.
    Reads the same at any width.
  - `.banded` — Tufte's model: the text column is offset left and notes **float** into
    a reserved right margin (`float`/`clear`, so consecutive notes stack instead of
    overlapping). Below the band width (62rem) there's no room to float, so it reverts
    to a normal centered column and behaves **identically to `.centered`** — the same
    collapse / click-any-opens-all disclosure.

  Pure CSS: counters (numbering), floats (placement), a checkbox toggle (mobile). Every
  split is a `--sn-*` custom property, so the proportions live in one place.

Both are opt-in — the default page stays a normal full-measure column with footnotes.
