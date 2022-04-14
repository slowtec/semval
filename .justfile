# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Format source code
fmt:
    cargo fmt --all

# Run clippy
check:
    cargo clippy --locked --workspace --bins --examples --tests -- -D warnings
    cargo clippy --locked --workspace --no-default-features --bins --examples --tests -- -D warnings
    cargo clippy --locked --workspace --all-features --bins --examples --tests -- -D warnings

# Fix lint warnings
fix:
    cargo fix --workspace --all-features --bins --examples --tests
    cargo clippy --workspace --all-features --bins --examples --tests --fix

# Run unit tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --all-features -- --nocapture

# Update depenencies and pre-commit hooks
update:
    rustup self update
    cargo install cargo-edit
    cargo upgrade --workspace
    cargo update
    pip install -U pre-commit
    pre-commit autoupdate

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files
