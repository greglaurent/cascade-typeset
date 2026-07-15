// Cascade — typography system for Typst.
//
// Re-exports the public modules. Typical usage:
//
//   #import "@local/cascade:0.1.0": layout, rhythm
//   #let l = layout.make()
//   #show: l.page
//   #show: l.markup
//   = Heading
//   Body with *bold* and _italic_ and `code`.
//
// Or pull a single module:
//
//   #import "@local/cascade:0.1.0": layout
//   #let l = layout.make(scale: "golden-ditonic")
//
// Modules:
//   layout — named callable components (headings, body, lists, quotes, etc.)
//            plus `page` and `markup` show rules. Top-level entry point.
//   font   — optical profile (tracking / leading / word-space as size-dependent
//            functions) and family + scale + profile bundles.
//   scale  — Mortensen-style typographic scale: f_i = f_0 · r^(i / n).
//   rhythm — vertical rhythm: baseline grid + spacing tokens derived from
//            scale + font + measure.
//   theme  — semantic color tokens (fg, bg, accent, …) with light/dark presets.

#import "layout.typ"
#import "font.typ"
#import "scale.typ"
#import "rhythm.typ"
#import "theme.typ"
