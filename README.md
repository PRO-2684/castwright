# castwright

> [!WARNING]
> This project is still in the early stages of development and is not yet ready for use.

🎥 Scripted terminal recording.

## Introduction

> [!NOTE]
> The name `castwright` is a portmanteau of `asciicast` and `playwright`.

Have you re-recorded the same session over and over again, either to hit the right speed or to avoid mistakes? Ever wanted to automate the process of recording terminal sessions, like shell scripts automate the process of running commands? Well, `castwright` is here to help.

## Features

- **Painless scripting**: `castwright` scripts are very intuitive and similar to an interactive shell, making it easy to understand, write, and maintain.

## Installation

TBD

## Usage

```shell
Usage: castwright [-i <input>] [-o <output>]

🎥 Scripted terminal recording.

Options:
  -i, --input       the path to the input file (castwright script `.cw`), or
                    stdin if not provided
  -o, --output      the path to the output file (asciicast `.cast`), or stdout
                    if not provided
  --help, help      display usage information
```

## Reference

See [`REFERENCE.md`](./REFERENCE.md) for a detailed reference of the `castwright` script format (`.cw`).

## TODO

- [x] Implement the `castwright` script parser.
- [x] Write to an asciicast file.
- [x] Terminal width and height detection.
- [ ] Actual command execution and output capture.
- [ ] Dry-run mode. (Print the commands to be executed without actually executing them)

## Credits

- [asciinema](https://asciinema.org)
- [autocast](https://github.com/k9withabone/autocast)
- [asciinema-scenario](https://github.com/garbas/asciinema-scenario)
