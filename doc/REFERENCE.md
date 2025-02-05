# `CastWright` Script Reference

## Introduction

> Conventionally, `.cwrt` is used as the file extension for CastWright scripts.

A CastWright script consists of two parts: the front matter and the body.

The [front matter](#front-matter), whose syntax is a subset of the standard YAML front matter, allows you to customize the metadata of the output asciicast, and can be skipped if you don't need it.

The body is line-based, with each line representing a single instruction, much like an interactive shell. The first character of the line determine the type of instruction, followed by instruction-specific argument(s). See [Instruction Types](#instruction-types) for a table of prefixes and their corresponding instruction types. Any line that does not start with one of those prefixes will result in an error (`UnknownInstruction`).

## Front Matter

The front matter must be enclosed by a pair of triple dashes (`---`). It consists of key-value pairs, where the key is a [String](#string) and the value's type depends on the key. The key-value pairs are separated by a colon (`:`), and each pair is on a separate line. The key is case-insensitive, and the value is case-sensitive. The front matter ends with another pair of triple dashes (`---`). Example:

```yaml
---
title: My Asciicast
shell: bash
idle: 5s
---
```

The following keys are supported in the front matter:

- `width`: Set the width of the terminal.
    - **Type**: [Integer](#integer).
    - **Default**: Current terminal width, or $80$ if not available.
- `height`: Set the height of the terminal.
    - **Type**: [Integer](#integer).
    - **Default**: Current terminal height, or $24$ if not available.
- `title`: Set the title of the asciicast.
    - **Type**: [LooseString](#loosestring).
    - **Default**: None.
- `shell`: Set the shell to be used for executing commands.
    - **Type**: [LooseString](#loosestring), which represents the shell executable.
    - **Default**: `bash`.
    - Provided shell must accept `-c` flag for executing commands.
- `quit`: Set the quit command to be used for exiting the shell. (Not implemented yet)
    - **Type**: [LooseString](#loosestring), which represents the quit command.
    - **Default**: `exit`.
- `idle`: Set the idle time limit for the asciicast.
    - **Type**: [Duration](#duration).
    - **Default**: None.
- `capture`: Set captured environment variables for the asciicast.
    - **Type**: A list of [String](#string).
    - **Default**: `["SHELL", "TERM"]`. (As [specified by asciinema](https://docs.asciinema.org/manual/asciicast/v2/#env))
    - Notes:
        - If you don't want to capture any environment variables, you can provide an empty list `[]`.
        - If the environment variable is not set or not valid unicode, it will be ignored.

Internally, front matter delimiters and key-value pairs are also treated as instructions.

## Instruction Types

| Prefix | Instruction Type |
| ------ | ---------------- |
| `$`    | [Command](#command) |
| `>`    | [Continuation](#continuation) |
| `@`    | [Configuration](#configuration) |
| `\s`   | [Empty line](#empty-line) |
| `#`    | [Comment](#comment) |
| `!`    | [Marker](#marker) |
| `%`    | [Print](#print) |
| `~`    | [Wait](#wait) |

### Command

A command instruction prints, executes in a separate shell session, and displays the output of a command. Usually, you'll use it the most in your script. Example:

```plaintext
$ echo "Hello, World!"
```

Note that each command is executed in a separate shell session, so you cannot define variables in one command and use them in another. This is a [known caveat](./CAVEATS.md#shell-session). However, CastWright implements a few built-in commands to help you work around this limitation:

- `cd`: Change the current working directory.
    - **Arguments**: A [LooseString](#loosestring) representing the path to change to.
    - **Example**: `$ cd ./path/to/directory`, `$ cd "directory with spaces"`

### Continuation

A continuation instruction is a continuation of a multi-line shell command. It must be used after a command instruction or another continuation instruction that ends with a backslash (`\`). Example:

```plaintext
$ echo "Multi-" \
> "line" \
> "command"
```

### Configuration

A configuration instruction configures the behavior of [command](#command) and [print](#print) instructions (Continuation of a command is considered the same instruction). It can be classified based on its scope as either [**persistent**](#persistent) or [**temporary**](#temporary). Below is a list of valid keywords for configuration instructions.

- `prompt`: Set the shell prompt to use in the asciicast output.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@prompt "$ "`
    - Note: For a cyan prompt, try setting `@@prompt "\u001b[36m$ \u001b[0m"`
- `secondary`/`secondary-prompt`: Set the secondary prompt to use in the asciicast output.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@secondary "> "`
    - Note: For a dimmed cyan secondary prompt, try setting `@@secondary "\u001b[2;36m> \u001b[0m"`
- `continuation`/`line-continuation`: Set the string to signify that the command continues on the next line.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@continuation " \\"`
    - Note: For a dimmed line continuation, try setting `@@continuation "\u001b[2m \\\u001b[0m"`
- `hidden`: Set whether the command should be executed silently.
    - **Parameter**: A [Boolean](#boolean), defaulting to `true` if omitted.
    - **Default**: `@@hidden false`
- `expect`: Set the expected exit status of the command. Does nothing to print instructions.
    - **Parameter**: `success`, `failure`, or `any`. If omitted, defaults to `success`.
    - **Default**: `@@expect success`
- `interval`: Set the typing interval between characters in a command.
    - **Parameter**: An [Duration](#duration).
    - **Default**: `@@interval 100ms`
- `start-lag`: Set the start lag. i.e. Additional delay after displaying the prompt, before printing the command for command instructions, or before printing the content for print instructions.
    - **Parameter**: An [Duration](#duration).
    - **Default**: `@@start-lag 0s`
- `end-lag`: Set the end lag. i.e. Additional delay after printing the command for command instructions, or after printing the content for print instructions.
    - **Parameter**: An [Duration](#duration).
    - **Default**: `@@end-lag 0s`

#### Persistent

A persistent config instruction starts with two `@`, and affects all subsequent instructions until another configuration instruction overrides it. It is useful for setting up the environment for the entire script when put at the beginning.

#### Temporary

A temporary config instruction starts with only one `@`, and affects only the next matched instruction. It is useful for changing the behavior of a single instruction without affecting the rest of the script.

### Empty

Empty instructions do nothing. They may take the form of an [empty line](#empty-line) or a [comment](#comment).

#### Empty Line

An empty line is a line that contains only whitespace characters. Useful for separating sections of your CastWright script.

#### Comment

A comment is a line that starts with a `#` character. You can use comments to document your script, but they have no effect on the output.

### Marker

A marker instruction marks a point in the asciicast output. It can be used to indicate the start of a new section or to provide a reference point for the viewer. Example:

```plaintext
! Section 1
```

### Print

A print instruction takes a [LooseString](#loosestring) and prints it together with a newline. Example:

```plaintext
% This will be printed as it is
% "  Printed with indent"
```

### Wait

A wait instruction introduces a delay to the asciicast for a specified [Duration](#duration). Example:

```plaintext
$ echo "First line"
~ 1s
$ echo "Second line"
```

Note that this instruction is not to be confused with the `sleep` shell command: the latter will pause the execution, while the former will only introduce a delay in the asciicast output.

Also, this instruction is not to be confused with the `start-lag` and `end-lag` configuration instructions: the latter two can introduce a delay **inside a line**, while the "wait" instruction introduces a delay **between lines**.

For example, if you use `start-lag` before a command instruction, the delay will be introduced between the prompt and the command. If you use `wait` before a command, the delay will be introduced between the previous line and the prompt.

## Argument Types

Note that all arguments will be trimmed of leading and trailing whitespace before being parsed.

### Boolean

A boolean argument can be either `true` or `false`.

### Integer

An integer argument is a sequence of digits. It can be positive or negative.

### Duration

A duration argument is a sequence of digits followed by a unit. The unit can be one of `s` (seconds), `ms` (milliseconds) or `us` (microseconds). If `0` is provided, the unit can be omitted.

### String

A string is a sequence of characters enclosed in double quotes (`"`). If you need to include a double quote in the string, you can escape it with a backslash (`\"`). If you need to include a backslash, you can escape it with another backslash (`\\`).

### LooseString

A loose string, if starting and ending with a `"` character, will be treated as a `String`. Otherwise, its raw value will be used. For example, `"123 "` will be parsed as `"123 "`, while `123 ` will be parsed as `"123"` (trimmed).
