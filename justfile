# cascade-typeset task runner -- `just <recipe>` (run `just` for the list)

# cascade version, read from the spec crate, stamped onto each distribution build.
version := `grep -m1 '^version' cascade/Cargo.toml | sed -E 's/.*"(.*)".*/\1/'`

# show all recipes
default:
    @just --list

# --- distribution ---
# Build the SHIPPED CSS distribution into dist/css (committed): the default compiled cascade.css +
# its modules -- the "batteries-included" artifact a consumer includes. This is cascade's release
# step; the CLI compiles once here, so consumers (and the site) never compile cascade themselves.
# Re-run when the spec changes, then commit the result. (dist/typst lands with the Typst renderer.)
dist:
    cargo run -p cascade-cli -- build --out dist/css
    rm -f dist/css/.cascade-manifest.json
    printf 'cascade %s\n' '{{version}}' > dist/css/VERSION
    @echo "built dist/css (cascade {{version}})"

# --- dev ---

test:
    cargo test

check:
    cargo check

fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets -- -D warnings
