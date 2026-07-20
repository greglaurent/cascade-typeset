# cascade — Architecture

This document is the contract. If any code contradicts it, the code is wrong. It exists because
this design has been re-derived and re-broken repeatedly; it is not up for re-litigation.

---

## What this library IS — the point (read first, never lose this)

cascade takes **one spec** — typography **and** theme — and **generates, for each medium, a
customizable typographic foundation**: the type and colour as **live, overridable, composable
primitives** (CSS custom properties + `calc()`; Typst config + calc). You then **design *with* that
output** — build your own layouts, components, sidenotes, margin notes, whatever — and you **customize
it**: override the inputs (base, fonts, scale, theme), compose your own design on top. Because the
primitives carry the *actual typography* — live calculations over the overridable inputs — you can
customize and override freely and the typography stays **valid and consistent across every medium, by
construction**. You never re-derive or hand-sync the type.

The output is therefore **neither** of the two wrong things:
- **not** a renderer you hand-author (that is the two-places-to-sync mess this library exists to
  *delete* — hand-writing CSS and Typst separately is the **failure state**, not a fallback);
- **not** a locked, dead generated file (the output is *meant* to be customized).

It is a **generated, customizable design system per medium** — correct no matter how you build on it.
One spec in; a customizable, always-valid typographic foundation out, in every renderer.

**Three layers (keep them straight):**
1. **Spec** — typography + theme, the one canonical source (values, calculations, roles, colours).
2. **Projection** — each renderer re-states the spec's values as live, overridable primitives in its
   medium. This is the part under the strict cross-renderer diff.
3. **Customization / design components** — what a *consumer* (or a renderer's own design layer) builds
   *with* the primitives: layouts, sidenotes, banded/centered, disclosure, chrome. These **consume**
   spec-projected values (typography + theme) and **never author their own** — so nothing drifts — but
   which components exist and how they're arranged/interact is design, not spec, and not under the diff.

If any proposal makes you hand-author a renderer, or bakes a value so customization can't recompute it,
it has broken the point above. That is the recurring failure; do not repeat it.

---

## The one rule

**One spec, N renderers, and it must be *impossible* for a renderer to render the wrong typography.**

The spec defines the typography. A renderer only re-states it in its own context. The moment a
renderer *decides* any typographic value — computes it, hand-writes it, or bakes it to a literal —
the invariant is already broken, whether or not a test happens to catch it. Correctness must be a
*property of the architecture*, not of anyone's discipline.

---

## The spec is the canonical calculation

The `cascade` crate is the single source of truth. It owns **everything that determines the
rendered result** — with no carve-outs:

- type scale (per-role size), leading, tracking, word-space, the optical model
- the reading measure
- vertical rhythm: unit, baseline, spacing tokens
- the document roles
- **colour and theme** (role × theme × mode) — in the spec exactly like size is

It is not merely a bag of definitions. It is the **canonical evaluator**: given a set of inputs,
`cascade::formula::*` (over the spec tokens) produces the one correct answer. That canonical
calculation is the reference every renderer is measured against.

If a typographic number or formula lives anywhere outside the spec, that is the bug.

---

## A renderer is a contextual emitter

A renderer emits exactly **two** things, and nothing else:

1. **Overridable inputs** — the knobs (base size, typeface, scale, theme, the optical tokens),
   exposed plainly so the consumer can change them.
2. **The spec's canonical calculations, live**, re-stated in the medium's syntax, reading those
   inputs.

The renderer owns **only context**: units, syntax, how it packages the inputs and the calc, which
knobs it exposes, delivery, and reactive switching. It **never** computes, authors, or bakes a
typographic value. Its entire job is: *how do I say the spec's value here, and which knobs do I
expose.*

- **CSS** context: custom properties (the inputs) + `calc()`/`clamp()` referencing them (the
  calculations) + selectors/classes/media-queries for reactive switching.
- **Typst** context: an overridable config (the inputs) + Typst calc/functions referencing it (the
  calculations). Typst has a runtime and a calc language — it emits calculations, **not** numbers.

---

## Overridability ⇒ always-valid typography

Because the output *contains the calculations* rather than frozen results, changing any input
re-derives through them and lands on correct typography every time. Only inputs are exposed;
everything else is the spec doing the math. The consumer can turn the knobs but **cannot reach an
invalid state**, because invalid states are not representable — there is no raw `line-height` knob to
desync from the rhythm; there is only `base`, and the rest derives.

---

## The guarantee: broken is impossible

The spec provides the canonical calculation. Each renderer's **emitted calculation is compared
against the spec's canonical calculation** — the same formula-graph over the same variables, modulo
context syntax. It must match the spec, and therefore the other renderer.

- A renderer that **authored** a value emits a calculation that doesn't match the canonical one → caught.
- A renderer that **baked** a value emits a literal where a calculation should be → caught, and it
  was never comparable in the first place.

The comparison is total (every input, every calculation, both sides) and needs **no oracle outside
the spec** and **no evaluate-to-compare hack**. The two renderer outputs are the same document
expressed twice; diffing them against the canonical spec (and each other) locates any divergence.

---

## Hard rules

Violating any of these is *the* bug this project keeps fighting. They are non-negotiable.

1. **NO authoring.** A renderer must never compute or hand-write a typographic value. Every value is
   `formula` projected into the renderer's syntax.
2. **NO baking.** A renderer must never collapse a calculation to a literal. The output carries the
   live calculation *and* its overridable inputs. Baking destroys both overridability and
   comparability — it is the single largest violation to remove.
3. **NO renderer-local constants or duplicated formulas.** A typographic number or a formula in a
   renderer is wrong by definition; it belongs in the spec.
4. **One source, one direction.** Add or change a value in the spec → it flows to every renderer
   automatically. A renderer never has a say in *what* a value is — only in *how* it is said.

---

## What is legitimately the renderer's (context / mechanism)

- Units and syntax (px/pt/em; CSS `calc`/`var` vs Typst calc).
- Packaging: CSS custom properties + selectors; Typst overridable config + functions.
- Which knobs to expose (a dark-mode toggle makes sense on the web, not in print).
- Delivery (`@font-face` vs `--font-path`) and reactive switching (classes, media queries).
- **Medium-only application of a shared value**, where there is genuinely no cross-renderer analogue:
  CSS `line-height` vs Typst `top/bottom-edge` box; CSS `font-size-adjust` vs a relative-em size.
  Both sides receive the **same spec value**; the guarantee here is *"same value in,"* verified at
  the medium level (box model / visual), **not** by the cross-renderer diff. This is the only place
  the diff guarantee does not reach — keep it minimal, explicit, and never let a *derivation* hide in
  it.

---

## Verification standard — every equation grounded

Every equation in the spec — `size_factor`, `lead0`/`leading`, `tracking`, `word_space`, `baseline`,
`rhythm_unit`/`spacing`, `measure_width`, `role_size`, `code_size`, and the font-measurement math —
is **foundational and verifiable, not chosen.** Each is cross-referenced against **each** major
typographic resource (Bringhurst, Butterick, Tracy, Tim Brown / modular scale, Capsize, WCAG, and any
other that bears on it) for both its **form** and its **constants**. The deliverable per equation is
the cross-reference itself: what each source says, where they agree, where they conflict — yielding a
grounded, cited equation, or one **explicitly marked ungroundable**.

Verify against the research. **Never** ask the author to confirm a value the literature settles;
**never** label a value "spec-chosen" to dodge grounding it. The spec is mostly **calculated** from a
tiny set of real inputs — `base_pt = 11` is the only author-confirmed one; every other optical
constant is agent-introduced and suspect until grounded, and several are calculations frozen into
magic numbers.

## Three paths — Digital / Print / Identical

Each equation resolves, by a **research-backed** decision, to one of three paths:

- **Identical** — medium-agnostic; both renderers emit the same calculation; the cross-renderer diff
  must match exactly. The default.
- **Digital** — the literature supports a screen-optimized value that legitimately differs from print.
- **Print** — the literature supports a print-optimized value that legitimately differs from digital.

A Digital/Print split is valid **only** if a major resource justifies the optimization; absent that,
it is Identical. "Optimized" is never a license for arbitrary difference. The diff-must-match
guarantee applies to **Identical**; on Digital/Print each renderer matches its own medium's grounded
value, and the split itself is cited.

## Current state — the violations to fix (the work)

- **`cascade-typst` bakes numbers.** The largest violation. It must emit an overridable Typst config
  + the formulas as Typst calc, exactly as CSS emits custom properties + `calc()`. The `f64`
  "evaluate to a number" projection is fine as an *internal check*, never as renderer output.
- **`cascade-css` hand-writes calc in the `optical` / `scale` / `rhythm` templates**, duplicating
  `formula`. Those templates die; the calc must be `formula` re-stated, not a hand-copied twin.
- **Renderer-local derivations** — measure→width (the `1em` bug), code sizing, rhythm unit, the size
  floor — are computed independently in each renderer today. They move into `formula`; both renderers
  re-state them.
- **The current parity test folds/bakes to compare** — a shadow that only exists *because* Typst
  bakes. Replace it with the real check: diff each renderer's emitted calculation against the spec's
  canonical calculation (and thus each other).

---

## Definition of done

- No literal typographic number and no formula lives in any renderer.
- Both renderers emit *overridable inputs + live calculations*; neither bakes.
- Changing any exposed input in either renderer's output yields correct typography.
- A single comparison diffs each renderer's calculation against the spec's canonical calculation; any
  divergence fails it and names the exact site. This comparison — not discipline, not review — is the
  guarantee.
