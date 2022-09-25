# SPDX-FileCopyrightText: slowtec GmbH
# SPDX-License-Identifier: CC0-1.0

# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Set up (and update) tooling
setup:
    # Ignore rustup failures, because not everyone might use it
    rustup self update || true
    # cargo-edit is needed for `cargo upgrade`
    cargo install just cargo-edit
    pip install -U pre-commit
    pre-commit install --hook-type commit-msg --hook-type pre-commit

# Upgrade (and update) dependencies
upgrade: setup
    pre-commit autoupdate
    cargo upgrade
    cargo update

# Format source code
fmt:
    cargo fmt --all

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Run build checks
check:
    cargo clippy --locked --workspace --all-targets -- -D warnings
    cargo clippy --locked --workspace --no-default-features --all-targets -- -D warnings
    cargo clippy --locked --workspace --all-features --all-targets -- -D warnings

# Fix code issues
fix:
    cargo fix --workspace --all-features --all-targets
    cargo clippy --workspace --all-features --all-targets --fix

# Run unit tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --all-features -- --nocapture
