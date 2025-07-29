# List all available commands
default:
    just --list

# Clean the build artifacts
clean:
    cargo clean

# Linting
clippy:
   cargo clippy --workspace --all-features --all-targets -- -D warnings

# Check formatting
check-fmt:
    cargo +nightly fmt --all -- --check

# Fix formatting
fmt:
    cargo +nightly fmt --all

# Test the project
test:
    cargo test --workspace --all-features --all-targets
    cargo run --example echo-tester

# Run all the checks
check:
    just check-fmt
    just clippy
    just test
