# castwright

> [!WARNING]
> This project is still in the early stages of development, with some core features missing or incomplete. Please refer to the [TODO](#todo) section for more information.

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

### Command Line Interface

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

### Castwright Script

A castwright script is a text file, conventionally with the `.cw` extension. It is line-based, with each line representing a single instruction. For example:

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

In addition, castwright provides various commands for customizing the produced asciicast, like typing speed or title. See [`REFERENCE.md`](./REFERENCE.md) for a detailed reference, or example castwright scripts under the [`tests`](./tests/) directory.

## TODO

- [x] Implement the `castwright` script parser.
- [x] Write to an asciicast file.
- [x] Terminal width and height detection.
- [ ] Better `pub` API.
- [ ] Dry-run mode. (Print the commands to be executed without actually executing them)
- [ ] Actual command execution and output capture.

## Credits

- [asciinema](https://asciinema.org)
- [autocast](https://github.com/k9withabone/autocast)
- [asciinema-scenario](https://github.com/garbas/asciinema-scenario)
