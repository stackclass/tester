# List all available commands
default:
    just --list

# Clean the build artifacts
clean:
    cargo clean --verbose

# Linting
clippy:
   cargo clippy --workspace --all-features --tests --bins --benches -- -D warnings

# Check formatting
check-fmt:
    cargo +nightly fmt --all -- --check

# Fix formatting
fmt:
    cargo +nightly fmt --all

# Test the project
test:
    RUST_BACKTRACE=1 cargo test --workspace --all-features --verbose

# Run all the checks
check:
    just check-fmt
    just clippy
    just test
