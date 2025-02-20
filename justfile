alias r := run
alias t := test

# Compile and run
run *args:
    cargo run --features="cli" -- {{args}}

# Run tests - drop-in replacement for `cargo test`
test *args:
    cargo test {{args}}
