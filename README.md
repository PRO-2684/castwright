# 🎥 CastWright

[![GitHub License](https://img.shields.io/github/license/PRO-2684/castwright?logo=opensourceinitiative)](https://github.com/PRO-2684/castwright/blob/main/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/PRO-2684/castwright/release.yml?logo=githubactions)](https://github.com/PRO-2684/castwright/blob/main/.github/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/PRO-2684/castwright?logo=githubactions)](https://github.com/PRO-2684/castwright/releases)
[![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/PRO-2684/castwright/total?logo=github)](https://github.com/PRO-2684/castwright/releases)
[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/PRO-2684/castwright/latest/total?logo=github)](https://github.com/PRO-2684/castwright/releases/latest)
[![Crates.io Version](https://img.shields.io/crates/v/castwright?logo=rust)](https://crates.io/crates/castwright)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/castwright?logo=rust)](https://crates.io/crates/castwright)

🎥 Scripted terminal recording.

## ℹ️ Introduction

> [!NOTE]
> The name `castwright` is a portmanteau of `asciicast` and `playwright`.

Have you re-recorded the same session over and over again, either to hit the right speed or to avoid mistakes? Ever wanted to automate the process of recording terminal sessions, like shell scripts automate the process of running commands? Well, CastWright is here to help.

## 🪄 Features

- **Fast**: CastWright is designed to be fast, with running time close to actually executing the given shell commands.
- **Efficient**: CastWright guarantees constant memory usage, by leveraging streaming I/O.
- **Preview**: Preview your asciicast automatically, saving you the time for executing `&& asciinema play name.cast` and the typing delays.
- **Intuitive**: CastWright scripts are very intuitive and similar to an interactive shell, making it easy to understand, write, and maintain.
- **Customization**: CastWright scripts provide various instructions for customizing the produced asciicast, like typing speed or title.
- **Reusability**: CastWright scripts can be reused across different environments, making it easy to share and collaborate.

## 🚀 Installation

If you have `cargo-binstall`, you can install this tool by running:

```shell
cargo binstall castwright
```

Otherwise, you can install it from source:

```shell
cargo install castwright
```

Pre-built binaries are available at [Releases](https://github.com/PRO-2684/castwright/releases).

## 📖 Usage

### Command Line Interface

```shell
Usage: castwright [-i <input>] [-o <output>] [-x]

🎥 Scripted terminal recording.

Options:
  -i, --input       the path to the input file (CastWright script `.cwrt`), or
                    stdin if not provided
  -o, --output      the path to the output file (asciicast `.cast`), or stdout
                    if not provided; If provided, preview mode will be enabled
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

## 🚫 Caveats

You can find a list of known caveats in [`CAVEATS.md`](./doc/CAVEATS.md#shell-session). Most notably, each command is executed in a separate shell session, which may not be ideal for some use cases.

## ✅ TODO

- [x] Implement the CastWright script parser.
- [x] Write to an asciicast file.
- [x] Terminal width and height detection.
- [x] Better `pub` API.
- [x] Run `cargo clippy`.
- [x] Integration tests.
- [x] Actual command execution and output capture.
    - [ ] Use a single shell session, instead of spawning a new one for each command.

## 🎉 Credits

- [asciinema](https://asciinema.org)
- [autocast](https://github.com/k9withabone/autocast)
- [asciinema-scenario](https://github.com/garbas/asciinema-scenario)
