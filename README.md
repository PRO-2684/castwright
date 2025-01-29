# CastWright

> [!WARNING]
> This project is still in the early stages of development, with some core features missing or incomplete. Please refer to the [TODO](#todo) section for more information.

🎥 Scripted terminal recording.

## Introduction

> [!NOTE]
> The name `castwright` is a portmanteau of `asciicast` and `playwright`.

Have you re-recorded the same session over and over again, either to hit the right speed or to avoid mistakes? Ever wanted to automate the process of recording terminal sessions, like shell scripts automate the process of running commands? Well, CastWright is here to help.

## Features

- **Painless scripting**: CastWright scripts are very intuitive and similar to an interactive shell, making it easy to understand, write, and maintain.

## Installation

TBD

## Usage

### Command Line Interface

```shell
Usage: castwright [-i <input>] [-o <output>] [-x]

🎥 Scripted terminal recording.

Options:
  -i, --input       the path to the input file (CastWright script `.cwrt`), or
                    stdin if not provided
  -o, --output      the path to the output file (asciicast `.cast`), or stdout
                    if not provided
  -x, --execute     execute and capture the output of shell commands
  -h, --help        display usage information
```

### CastWright Script

A CastWright script is a text file, conventionally with the `.cwrt` extension. It is line-based, with each line representing a single instruction. For example:

```plaintext
$ echo "Hello, World!"
```

Would output an asciicast recording of the following:

```plaintext
$ echo "Hello, World!"
Hello, World!
```

For multiline commands, use the `>` prefix and `\` suffix, as you would in an interactive shell. For example:

```plaintext
$ echo "Multi-" \
> "line" \
> "command"
```

Would output an asciicast recording of the following:

```plaintext
$ echo "Multi-" \
> "line" \
> "command"
Multi- line command
```

In addition, CastWright provides various instructions for customizing the produced asciicast, like typing speed or title. See [`REFERENCE.md`](./doc/REFERENCE.md) for a detailed reference, or example CastWright scripts under the [`tests`](./tests/) directory.

## Caveats

You can find a list of known caveats in [`CAVEATS.md`](./doc/CAVEATS.md#shell-session). Most notably, each command is executed in a separate shell session, which may not be ideal for some use cases.

## TODO

- [x] Implement the CastWright script parser.
- [x] Write to an asciicast file.
- [x] Terminal width and height detection.
- [x] Better `pub` API.
- [x] Run `cargo clippy`.
- [x] Integration tests.
- [x] Actual command execution and output capture.
    - [ ] Use a single shell session, instead of spawning a new one for each command.

## Credits

- [asciinema](https://asciinema.org)
- [autocast](https://github.com/k9withabone/autocast)
- [asciinema-scenario](https://github.com/garbas/asciinema-scenario)
