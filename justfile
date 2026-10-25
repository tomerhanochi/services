# File-server development and verification.
set shell := ["bash", "-euo", "pipefail", "-c"]
# Load the command modules for services and their crates.
mod bin

# List the available root commands.
default:
    @just --list

# Format all Rust source files.
fmt:
    cargo fmt --all

# Format and type-check the workspace.
check:
    cargo fmt --all --check
    cargo check --workspace --all-targets

# No database required; database integration tests are explicitly ignored.
test:
    cargo test --workspace --all-targets

# Requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database.
test-all:
    cargo test --workspace --all-targets -- --include-ignored --test-threads=1
