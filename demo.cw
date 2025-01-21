# Comments start with `#`, which won't have any effect on the outcome

# To print some content as it is, prefix with `%`
% This will be printed as it is
# Output:
# This will be printed as it is

# Metadata or persistent configuration (`@@`)
# Sets configuration for all FOLLOWING commands
# Set the terminal width, default is current terminal width
@@width auto
# Set the terminal height, default is current terminal height
@@height auto
# Title of the asciicast (no point setting this using temporary configuration)
@@title Castwright Script
# Terminal idle timeout
@@idle 1s
# Delay between each character
@@delay 100ms
# ...

# Temporary configuration (`@`): Configuration for this command only
@hidden
# The following command will be executed but not printed

# Shell command (`$`, an extra space is recommended)
$ echo "Hello, World!"
# No output

# This command will be printed and executed
$ echo "Hello, World again!"
# Output:
# $ echo "Hello, World again!"
# Hello, World again!

# Marker (`!`)
!Multiline

# Start a multiline command with a regular command
# Use `>` as prefix to continue
$ echo "Multi-" \
> "line" \
> "command"
# Output:
# $ echo "Multi-" \
# > "line" \
# > "command"
# Multi- line command
