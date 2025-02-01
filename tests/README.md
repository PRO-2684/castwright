# Tests

This directory contains tests for the project.

## Input-Output tests, without execution

- [`input`](./input/): Input files.
- [`output`](./output/): Expected output files.

## Success or failure tests, with execution

> [!NOTE]
> These tests are only tested on linux, since they use the `bash` shell and symbolic links.

- [`success`](./success/): Input files that should execute successfully.
- [`failure`](./failure/): Input files that should fail to execute.
