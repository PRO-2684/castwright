# Caveats

This section lists known issues and caveats of CastWright.

## Shell Session

### Issue

> [!NOTE]
> CastWright implements some builtin shell commands to help you work around this limitation. See [REFERENCE.md](./REFERENCE.md#command) for more information.

Each command in a CastWright script is executed in a separate shell session. This means that changes to the environment, like setting environment variables, are not preserved between commands. This may not be ideal for some use cases.

### Workaround

Use multi-line shell commands to group related commands together. For example, you can change the following:

```plaintext
$ export MY_VAR="Hello, World!"
$ echo $MY_VAR
$ unset MY_VAR
```

To:

```plaintext
$ export MY_VAR="Hello, World!" \
> && echo $MY_VAR \
> && unset MY_VAR
# The above line can be omitted, since current implementation does not preserve the shell session. However, it is recommended to include it for:
# Clarity and maintainability
# Easy transition to a future implementation that preserves the shell session
# Copy-pasting the commands to a shell script
```

This way, related commands are executed in the same shell session.

### Reason

Currently for simplicity and ease of implementation, CastWright uses a similar API like `std::process::Command` to invoke the given shell and pass the command to it as an argument after `-c`. This means that each command is executed in a separate shell session.

To maintain a shell session across multiple commands, we would need to invoke the shell once and pass the commands to it one by one. Passing commands to the shell process would be easy, but determining when the previous command has finished executing would be difficult. See [the following section](#possible-solutions) for some solutions by other projects.

### Possible Solutions

#### Expecting the Shell Prompt

This is a technique widely used by the community, including [autocast](https://github.com/k9withabone/autocast), [rexpect](https://github.com/rust-cli/rexpect) and [pexpect](https://pexpect.readthedocs.io/en/stable/). The idea is to wait for the shell prompt to appear before sending the next command. This is not ideal because:

1. It is not universal. Different shells have different prompts.
2. It is reliable most of the time, but not always. False positives can occur if the prompt appears in the output of the command. For example, if you're using [Nu Shell](https://www.nushell.sh/), the default prompt is `~> `, which can appear in the output of a command.
3. It is not elegant. It is a rather hacky solution in my point of view that can break easily.
4. Cannot capture return codes.

#### Integrate a Modified Shell

Another solution is to modify a shell written in Rust and integrate it into CastWright. This way, we don't have to worry about the prompt and can maintain a shell session across multiple commands. This is a more reliable solution, but:

1. It requires a lot of work, apparently. We need to modify the shell to accept commands programmatically and integrate it into CastWright.
2. It makes CastWright bloated. We don't want to include a shell in CastWright. We want to keep it simple and lightweight.
3. It does not work with other shells and can incur learning costs for users.

#### Implement Common Builtin Shell Commands (Current Solution)

Another solution is to implement common builtin shell commands, like `cd`. This way, we can simulate a shell session without actually maintaining one. This is a more reliable solution, but:

1. It is not universal. Different shells have different commands, although most of them are similar.
2. Which commands to implement should be decided.

#### Expecting OSC 133/633 Escape Sequence

Very similar to [Expecting the Shell Prompt](#expecting-the-shell-prompt), but should be more robust and can capture return codes (OSC 163 only). Requires the shell to support the feature. See relevant documentations for details: [OSC 133 (iTerm2's documentation on Proprietary Escape Codes)](https://iterm2.com/documentation-escape-codes.html#FTCS_PROMPT:~:text=s%20source%20code.-,FTCS_PROMPT,-OSC%20133%20%3B%20A), [OSC 633 (VSCode Terminal Shell Integration)](https://code.visualstudio.com/docs/terminal/shell-integration#_vs-code-custom-sequences-osc-633-st).

## Contributing

[`src/shell.rs`](../src/shell.rs) contains the implementation of executing shell commands. You may want to start there and define a new structure like `ShellSession` if you are interested in contributing a solution to this issue.
