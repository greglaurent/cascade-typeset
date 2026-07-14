CASCADE — BRAND ASSETS
======================

The mark is a C that resolves into an E through nested concentric strokes — a
typesetting mark about recursion and containment. It is neutral and geometric on
purpose. The single oxblood stroke at the center is rubrication: the medieval
convention of marking the significant passage in red. It appears where the
recursion terminates. This is the ONLY saturated color in the brand.

PALETTE — four tokens, two modes. Nothing else exists.
                     light       dark
  --brand-ink        #1F1E1C     #EDE9E0
  --brand-paper      #F2EEE5     #1A1815
  --brand-accent     #7A2E28     #E09A93
  --brand-mute       #9A9388     #7A7469
  ink/paper contrast: 15.8:1 light · 14.9:1 dark.

ASSIGNMENTS
  mark, all strokes .................. ink
  mark, innermost E stroke ........... accent   (PRIMARY LOCKUP ONLY)
  wordmark CASCADE ................... ink
  tagline TYPESETTING SYSTEM ......... mute
  background ......................... paper

FILES  (all SVG, fully offline — see NOTE ON TYPE)
  /mark      cascade-mark-{light,dark}.svg            neutral — use everywhere repeated
             cascade-mark-{light,dark}-accent.svg     accent center — primary use only
  /lockups   cascade-lockup-stacked-{light,dark}[-accent].svg
             cascade-lockup-horizontal-{light,dark}[-accent].svg
  /wordmark  cascade-wordmark-{light,dark}.svg        no accent, ever
  /favicon   cascade-favicon-{light,dark}.svg         drawn separately at 16px:
                                                      outer square + one inner rule,
                                                      no E arms, full stroke weight
  /specs     cascade-clearspace.svg                   clear space >= x (one nesting step)
             cascade-minsize.svg                      full mark to 24px; favicon at <=16px

RULES
  1. The accent stroke appears ONLY in the primary lockup (splash, homepage hero,
     README header). Anywhere small or repeated — favicon, nav, footer, avatar —
     use the all-neutral mark.
  2. Never warm the ink toward brown. --brand-ink is warm-neutral; any added chroma
     at that lightness reads as brown and kills it.
  3. The tagline is WARM gray (--brand-mute), never a blue-gray or system gray.
  4. Do not tint the wordmark. Never apply the accent to the wordmark or tagline.
  5. Reversed (dark) is a straight token swap — same geometry, no weight compensation.

FORBIDDEN
  Drop shadows · outlines · gradients · rotation · stretching · any color outside the
  four tokens · a second accent color "for variety" · the lockup on a photograph.

NOTE ON TYPE & COLOR
  Every file is fully offline and self-contained — no fonts, no @import, no network.
  The wordmark CASCADE and the tagline are the real Jost letterforms (weights 400 /
  500) converted to vector outlines, so they render identically everywhere, including
  <img> tags and design tools that ignore CSS. Spec-sheet labels are JetBrains Mono,
  also outlined.
  Colors are baked directly onto each shape as fill/stroke attributes (the token hex
  above), rather than CSS variables — CSS custom properties silently fail in <img>
  and in many SVG importers (Figma, Illustrator). To recolor, find-and-replace the
  hex. Token → hex mapping is the PALETTE table above; light and dark files use the
  two columns respectively.
