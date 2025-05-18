# Dev notes

## Available `just` recipes

```bash
$ jiu -l
CastWright: 🎥 Scripted terminal recording.

Available recipes:
  build/b            # Build release binary
  run/r *rest        # Compile and run
  test/t *rest       # Run tests
  version/v ?version # Set or get version
  ```

## <ins>B</ins>uild release binary

- Via `jiu`: `jiu b`
- Via `cargo`: `cargo build --release --bin castwright --features="cli"`

## Set or get <ins>v</ins>ersion

- Via script
    - Get: `./scripts/version.sh`
    - Set: `./scripts/version.sh 0.1.0`
- Via `jiu`
    - Get: `jiu v`
    - Set: `jiu v 0.1.0`

## Compile and <ins>r</ins>un

- Via `jiu`: `jiu r -h`
- Via script: `./scripts/run.sh -h`
- Via `cargo`: `cargo run --features="cli" -- -h`

## <ins>T</ins>ests

- Via `jiu`: `jiu t`
- Via VSCode: Run `workbench.action.tasks.test` task
- Via `cargo`: `cargo test`
