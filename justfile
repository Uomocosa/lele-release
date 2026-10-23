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

# Fail fast when the installed binary is not the version this repo expects.
tools:
    test "$(lele-release --version)" = "lele-release 0.1.0"

# Run the tool without installing it.
run *args:
    cargo run --quiet -- {{args}}
