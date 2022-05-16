# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Set up (and update) tooling
setup:
    rustup self update
    cargo install \
        cargo-edit \
        trunk
    pip install -U pre-commit
    pre-commit autoupdate
    #pre-commit install --hook-type commit-msg --hook-type pre-commit

# Format source code
fmt:
    cargo fmt --all

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files

# Upgrade (and update) depenencies
upgrade:
    cargo upgrade --workspace
    cargo update

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
