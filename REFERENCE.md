# `castwright` Script Reference

## Introduction

> Conventionally, `.cw` is used as the file extension for `castwright` scripts.

A `castwright` script is line-based, with each line representing a single instruction, much like a shell script. The first or first two characters of the line determine the type of instruction, followed by instruction-specific argument(s). Below is a table of prefixes and their corresponding instruction types.

| Prefix | Instruction Type |
| ------ | ---------------- |
| `@@`   | [Persistent configuration](#persistent) |
| `@`    | [Temporary configuration](#temporary) |
| `%`    | [Print](#print) |
| `!`    | [Marker](#marker) |
| `$`    | [Command](#command) |
| `>`    | [Continuation](#continuation) |
| `#`    | [Comment](#comment) |
| `\s`   | [Empty line](#empty-line) |

Any line that does not start with one of the above prefixes will result in an error (`UnknownInstruction`).

## Instruction Types

### Configuration

A configuration instruction configures the behavior of the output or other instructions. It can be classified based on its effect as either [**metadata**](#metadata) or [**directive**](#directive), or based on its scope as either [**persistent**](#persistent) or [**temporary**](#temporary). For simplicity, we will skip the "configuration" word and refer to these instructions as metadata, directive, persistent, or temporary instructions. Below is a list of valid keywords for configuration instructions.

- `width`: Set the width of the terminal.
    - **Effect**: Metadata; **Scope**: Persistent.
    - **Parameter**: A positive [Integer](#integer) or `auto`. If `auto` or `0`, the width will be determined by the width of current terminal.
    - **Default**: `@@width auto`
- `height`: Set the height of the terminal.
    - **Effect**: Metadata; **Scope**: Persistent.
    - **Parameter**: A positive [Integer](#integer) or `auto`. If `auto` or `0`, the height will be determined by the height of current terminal.
    - **Default**: `@@height auto`
- `title`: Set the title of the asciicast.
    - **Effect**: Metadata; **Scope**: Persistent.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@title Castwright Script`
- `shell`: Set the shell to be used for executing commands.
    - **Effect**: Metadata; **Scope**: Persistent.
    - **Parameter**: A [LooseString](#loosestring), which represents the shell executable.
    - **Default**: `@@shell bash`
- `quit`: Set the quit command to be used for exiting the shell.
    - **Effect**: Metadata; **Scope**: Persistent.
    - **Parameter**: A [LooseString](#loosestring), which represents the quit command.
    - **Default**: `@@quit exit`
- `idle`: Set the idle time limit for the asciicast.
    - **Effect**: Directive; **Scope**: Persistent.
    - **Parameter**: A [Duration](#duration).
    - **Default**: `@@idle 5s`
- `prompt`: Set the shell prompt to use in the asciicast output.
    - **Effect**: Directive; **Scope**: Persistent or Temporary.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@prompt "$ "`
- `secondary-prompt`: Set the secondary prompt to use in the asciicast output.
    - **Effect**: Directive; **Scope**: Persistent or Temporary.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@secondary-prompt "> "`
- `line-split`: Set the string to signify a line split in a multiline command.
    - **Effect**: Directive; **Scope**: Persistent or Temporary.
    - **Parameter**: A [LooseString](#loosestring).
    - **Default**: `@@line-split " \\"`
- `hidden`: Set whether the command should be executed silently.
    - **Effect**: Directive; **Scope**: Persistent or Temporary.
    - **Parameter**: An [OptionalBoolean(true)](#optionaldefault).
    - **Default**: `@@hidden false`
- `delay`: Set the typing delay between characters in a command.
    - **Effect**: Directive; **Scope**: Persistent or Temporary.
    - **Parameter**: An [Duration](#duration).
    - **Default**: `@@delay 100ms`

#### Metadata

A metadata instruction provides information about the asciicast output, such as the `width` and `height` of the terminal or the `title` of the asciicast.

#### Directive

A directive instruction changes the behavior of instructions, such as the typing `delay` or whether a command should be `hidden`.

#### Persistent

A persistent instruction affects all subsequent instructions until another configuration instruction overrides it. It is useful for setting up the environment for the entire script when put at the beginning.

#### Temporary

A temporary instruction affects only the next instruction (that is not a configuration instruction). It is useful for changing the behavior of a single instruction without affecting the rest of the script.

### Print

A print instruction takes a [LooseString](#loosestring) and prints it. It does not automatically add a newline character at the end, so you need to include it if you want one.

### Marker

A marker instruction marks a point in the asciicast output. It can be used to indicate the start of a new section or to provide a reference point for the viewer.

### Command

A command instruction prints, executes, and displays the output of a command. Usually, you'll use it the most in your script.

### Continuation

A continuation instruction is a continuation of a multi-line shell command. It must be used after a command instruction or another continuation instruction that ends with a backslash (`\`).

### Empty

Empty instructions do nothing. They may take the form of an [empty line](#empty-line) or a [comment](#comment).

#### Empty Line

An empty line is a line that contains only whitespace characters. Useful for separating sections of your script.

#### Comment

A comment is a line that starts with a `#` character. You can use comments to document your script, but they have no effect on the output.

## Argument Types

Note that all arguments will be trimmed of leading and trailing whitespace before being parsed.

### Boolean

A boolean argument can be either `true` or `false`.

### Integer

An integer argument is a sequence of digits. It can be positive or negative.

### Duration

A duration argument is a sequence of digits followed by a unit. The unit can be one of `s` (seconds), `ms` (milliseconds) or `us` (microseconds).

### String

A string is a sequence of characters enclosed in double quotes (`"`). If you need to include a double quote in the string, you can escape it with a backslash (`\"`). If you need to include a backslash, you can escape it with another backslash (`\\`).

### LooseString

A loose string, if starting and ending with a `"` character, will be treated as a `String`. Otherwise, its raw value will be used. For example, `"123 "` will be parsed as `"123 "`, while `123 ` will be parsed as `"123"` (trimmed).

### Optional\*(default)

An optional argument can be of given type or omitted. If omitted, the `default` value will be used. For example, `OptionalBoolean(true)` means the argument can be `true`, `false`, or omitted, with `true` as the default value.
