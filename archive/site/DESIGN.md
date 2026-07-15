# Site — Design System

This is the **website's** visual design: the viewer chrome and the brand identity. It is
**not** part of the cascade typesetting library — the site *consumes* cascade (loads its
CSS, iframes its specimens) and never the reverse.

Two token families live here, both viewer-only:

- **`--brand-*`** — the brand identity (mark, wordmark, favicon).
- **`--ui-*`** — the chrome (header, panels, tabs, buttons).

The site borrows exactly two values from the library — `--ct-bg` (the iframe ground) and
`--ct-accent` (the focus ring) — pulled live from the library's generated `theme.css`, not
copied. The document theme (`--ct-*`) is documented with the library, in `tools/tokens.mjs`.

The shared oxblood accent is a deliberate visual echo across brand, chrome, and document —
not a reason to store them together.

---

## 1. Brand ✅ `assets/`

```
                     light      dark
--brand-ink          #1F1E1C    #EDE9E0
--brand-paper        #F2EEE5    #1A1815
--brand-accent       #7A2E28    #E09A93
--brand-mute         #9A9388    #7A7469
```

| element | colour |
|---|---|
| mark (all strokes) | `--brand-ink` |
| mark — innermost E stroke, **primary lockup only** | `--brand-accent` |
| wordmark `CASCADE` | `--brand-ink` |
| tagline `TYPESETTING SYSTEM` | `--brand-mute` |
| lockup background | `--brand-paper` |

ink/paper: 15.8:1 light, 14.9:1 dark.

**Rules.** The accent stroke appears **only** in the primary lockup (splash / homepage hero /
README header). Everywhere small or repeated — favicon, nav, footer — use the all-neutral
mark. Draw a **separate 16px favicon**: outer square, one inner rule, no E arms. The lockup
mark does not scale down.

**Assets** (`assets/` — all self-contained SVG; colours baked as hex, not CSS vars, so they
render in `<img>` and design tools. Find-and-replace the hex to recolour; light/dark use the
two palette columns.):

| file | use |
|---|---|
| `mark/cascade-mark-{light,dark}.svg` | neutral mark — everywhere repeated (nav, footer, avatar) |
| `mark/cascade-mark-{light,dark}-accent.svg` | accent-centre mark — **primary lockup only** |
| `lockups/cascade-lockup-{stacked,horizontal}-{light,dark}[-accent].svg` | full lockups |
| `wordmark/cascade-wordmark-{light,dark}.svg` | CASCADE wordmark — never accented |
| `favicon/cascade-favicon-{light,dark}.svg` | 16px favicon (neutral) — wired in `index.html` |
| `specs/` | clear-space (≥ one nesting step) + min-size (full mark → 24px; favicon ≤ 16px) |

The mark is a C resolving into an E through nested concentric strokes; the single oxblood
stroke at the centre is rubrication — the only saturated colour in the brand, and it appears
**only** in the primary lockup. Verified: the accent hex is present only in the `-accent`
files; favicon / wordmark / neutral marks are pure ink. Dark is a straight token swap. Never
tint the wordmark, warm the ink toward brown, or add a second accent.

---

## 2. Chrome ✅ `site/index.html`

```
                     light      dark
--ui-bg              #EDEAE3    #100E0B
--ui-fg              #2A2723    #DDD8CF
--ui-bg-panel        #F6F2E9    #1A1713
--ui-border          #C9C3B8    #3D3830
--ui-border-strong   #9A9388    #565046
--ui-shadow          #2A272340  #00000080
```

`--ui-bg` is **darker** than the document paper — the frame recedes, the specimen sits on it
like paper on a desk. `--ui-bg-panel` **is** the document paper, so floating panels sit at the
document's plane.

| component | resting | active / hover |
|---|---|---|
| header bg/fg | `--ui-bg` / `--ui-fg` | — |
| header border-bottom | `--ui-border` | — |
| title `small` | `--ui-fg` @ 0.55 | — |
| dropdown summary | border `--ui-border-strong` | **open:** bg `--ui-fg`, fg `--ui-bg-panel` |
| panel | bg `--ui-bg-panel`, border `--ui-border-strong`, shadow `--ui-shadow` | — |
| tabs | border `--ui-border-strong`, bg transparent | **selected:** bg `--ui-fg`, fg `--ui-bg-panel` |
| buttons | border `--ui-border-strong`, bg transparent | hover: border `--ui-fg`; disabled: opacity 0.45 |
| theme toggle | border `--ui-border-strong`, SVG `currentColor` | hover: border `--ui-fg` |
| iframe | bg `--ct-bg` (library) | — |
| **focus ring** | **`--ct-accent` (library), 2px, offset 2px** | — |

Active states use ink/paper inversion, **never the accent**. The focus ring is the only accent
in the chrome — that's what makes it unmistakable.

**Mechanism.** `data-theme` on `<html>` drives the palette; chrome and the borrowed library
tokens flip from one switch. `color-scheme: light dark` stays declared **only** so scrollbars
and form controls render — it does not drive the palette.
