# Dev notes

## Compile and run

Via `cargo`:

```bash
$ cargo run --features="cli" -- -h
Usage: castwright ...
```

Via script:

```bash
$ ./scripts/run.sh -h
Usage: castwright ...
```

Via `just`:

```bash
$ just r -h
Usage: castwright ...
```

## Test

Via `cargo`:

```bash
cargo test
```

Via VSCode:

- Run `workbench.action.tasks.test` task

Via `just`:

```bash
just t
```
