# cascade-typeset — one typography system, two renderers (Typst + CSS).
#
# `@local/cascade:0.1.0` (so `#import "@local/cascade:0.1.0"` resolves to this repo)
# is provided declaratively by the nixos config (modules/home/typst.nix). On a
# non-nix machine, symlink cascade-typst/ into Typst's local-package namespace:
#   ln -s "$PWD/cascade-typst" ~/.local/share/typst/packages/local/cascade/0.1.0

# Default recipe: list available recipes.
default:
    @just --list

# Regenerate the token-driven parts of both renderers from tools/tokens.mjs.
gen:
    deno run --allow-read --allow-write tools/gen.mjs

# Fidelity checks: generated files in sync with tokens + Typst formulas vs the model.
verify:
    deno run --allow-read --allow-write --allow-run --allow-env tools/verify.mjs

# Type-check the toolchain (deno check — Node types built-in, no @types/node needed).
typecheck:
    deno check tools/tokens.mjs tools/gen.mjs tools/verify.mjs serve.ts

# Compile the Typst example to a PDF (viewable via `just serve` → Typst tab).
pdf:
    typst compile --root . cascade-typst/sample.typ cascade-typst/sample.pdf
    @echo "built cascade-typst/sample.pdf"

# Serve both examples (CSS/Typst viewer) at localhost:8175 — run `just pdf` first.
serve:
    deno run --allow-net --allow-read --allow-write --allow-run --allow-env serve.ts
