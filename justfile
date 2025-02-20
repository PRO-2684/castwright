alias r := run
alias t := test

# Bump version
bump:
    ./scripts/bump-version.sh

# Compile and run
run *args:
    cargo run --features="cli" -- {{args}}

# Run tests - drop-in replacement for `cargo test`
test *args:
    cargo test {{args}}
