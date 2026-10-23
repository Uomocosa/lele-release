set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Toolchain: rust-toolchain.toml (nightly-2026-09-01, matching the fleet).
# `status` and `release` live in the uncommitted justfile at the fleet root.

build:
    cargo build --all-targets --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt -- --check

test:
    cargo nextest run --all-targets --all-features

lint:
    lele-lint .

# Install the release binary from this checkout, honouring Cargo.lock.
install:
    cargo install --locked --force --path .

# Install the git filter on this machine: writes ~/.config/git/lele-pins.gitconfig
# from pins.gitconfig.template and adds one includeIf to ~/.gitconfig, then
# reports which repos still miss the committed .gitattributes line.
setup: install
    lele-release --root .. setup

# Install the prek hooks (fmt/clippy/lint on commit, tests on push).
hooks:
    prek install -t pre-commit -t pre-push

# Show which binary each tool resolves to and its version. A `<fleet>/bin` shim
# (written by `lele-release setup`) builds the sibling checkout; set
# LELE_USE_INSTALLED=1 to use the installed binary instead.
tools:
    @for tool in lele-release; do printf '%-24s %s  %s\n' "$tool" "$(command -v "$tool")" "$("$tool" --version)"; done

# Run the tool without installing it.
run *args:
    cargo run --quiet -- {{args}}
