# Dev notes

## Available `just` recipes

```bash
$ just --list
Available recipes:
    build      # Build release binary [alias: b]
    bump       # Bump version [alias: v]
    run *args  # Compile and run (debug) [alias: r]
    test *args # Run tests - drop-in replacement for `cargo test` [alias: t]
```

## <u>B</u>uild release binary

- Via `just`: `just b`
- Via `cargo`: `cargo build --release --bin castwright --features="cli"`

## Bump <u>v</u>ersion

- Via script: `./scripts/bump-version.sh`
- Via `just`: `just v`

## Compile and <u>r</u>un

- Via `just`: `just r -h`
- Via script: `./scripts/run.sh -h`
- Via `cargo`: `cargo run --features="cli" -- -h`

## <u>T</u>ests

- Via `just`: `just t`
- Via VSCode: Run `workbench.action.tasks.test` task
- Via `cargo`: `cargo test`
