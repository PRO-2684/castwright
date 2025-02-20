# Dev notes

## Available `just` recipes

```bash
$ just --list
Available recipes:
    bump       # Bump version
    run *args  # Compile and run [alias: r]
    test *args # Run tests - drop-in replacement for `cargo test` [alias: t]```
```

## Compile and run

- Via `just`: `just r -h`
- Via script: `./scripts/run.sh -h`
- Via `cargo`: `cargo run --features="cli" -- -h`

## Test

- Via `just`: `just t`
- Via VSCode: Run `workbench.action.tasks.test` task
- Via `cargo`: `cargo test`

## Bump version

- Via script: `./scripts/bump-version.sh`
- Via `just`: `just bump`
