# cascade-typeset — one typography system, two renderers (Typst + CSS).
#
# `@local/cascade:0.1.0` (so `#import "@local/cascade:0.1.0"` resolves to this repo)
# is provided declaratively by the nixos config (modules/home/typst.nix); on a
# non-nix machine, run `just link`.

# Default recipe: list available recipes.
default:
    @just --list

# Symlink the Typst library into Typst's local-package namespace so
# `#import "@local/cascade:0.1.0"` resolves. Idempotent. NO-OP on NixOS: there the
# symlink is owned declaratively by the nixos config (modules/home/typst.nix), and
# creating it here would fight home-manager activation ("would be clobbered").
link:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -e /run/current-system/nixos-version ] || grep -qs '^ID=nixos' /etc/os-release; then
        echo "NixOS detected — skipping: @local/cascade:0.1.0 is managed by the nixos"
        echo "config (modules/home/typst.nix). Nothing to do."
        exit 0
    fi
    mkdir -p ~/.local/share/typst/packages/local/cascade
    ln -sfn "{{justfile_directory()}}/cascade-typst" ~/.local/share/typst/packages/local/cascade/0.1.0
    echo "linked @local/cascade:0.1.0 → cascade-typst/"

# Regenerate the token-driven parts of both renderers from tools/tokens.mjs.
gen:
    deno run --allow-read --allow-write tools/gen.mjs

# Fidelity checks: generated files in sync with tokens + Typst formulas vs the model.
verify:
    deno run --allow-read --allow-write --allow-run --allow-env tools/verify.mjs

# Type-check the toolchain (deno check — Node types built-in, no @types/node needed).
typecheck:
    deno check tools/tokens.mjs tools/gen.mjs tools/verify.mjs site/serve.ts

# Compile the Typst example to a PDF (viewable via `just serve` → Typst tab).
pdf:
    typst compile --root . cascade-typst/sample.typ cascade-typst/sample.pdf
    @echo "built cascade-typst/sample.pdf"

# Serve both examples (CSS/Typst viewer) at localhost:8175 — run `just pdf` first.
serve:
    deno run --allow-net --allow-read --allow-write --allow-run --allow-env site/serve.ts
