# Contributing to CastWright

First off, thank you for considering contributing to CastWright! It's people like you that make this project great.

## How Can I Contribute?

### Reporting Bugs

If you find a bug, please report it by opening an issue on our [GitHub repository](https://github.com/PRO-2684/castwright/issues). Include as much detail as possible to help us understand and reproduce the issue.

### Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing feature, please open an issue on our [GitHub repository](https://github.com/PRO-2684/castwright/issues). Describe your idea in detail and explain why you think it would be beneficial.

### Submitting Pull Requests

> [!NOTE]
> Specifically, pull requests addressing [known caveats](./CAVEATS.md) are highly appreciated.

1. **Fork the repository**: Click the "Fork" button at the top right of the repository page.
2. **Clone your fork**:

    ```sh
    git clone https://github.com/YOUR-USERNAME/castwright.git
    cd castwright
    ```

3. **Create a new branch**:

    ```sh
    git checkout -b my-feature-branch
    ```

4. **Make your changes**: Implement your feature or fix the bug.
5. **Commit your changes**:

    ```sh
    git commit -am 'Add new feature'
    ```

6. **Push to your branch**:

    ```sh
    git push origin my-feature-branch
    ```

7. **Open a pull request**: Go to the original repository and click the "New pull request" button. Select your branch from the dropdown and submit the pull request.

### Coding Guidelines

- Run `cargo fmt` to format your code.
- Write clear, concise commit messages. Most of the time, you should follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.
- Add comments to your code where necessary, especially doc comments for public functions and modules. (This project has `#![deny(missing_docs)]` enabled.)
- Write tests for new features and bug fixes.
- Run `cargo clippy` and `cargo test` to ensure your changes do not introduce new issues.

### Documentation

Checkout [`DEV.md`](./DEV.md) for more information on how to compile, run, and test the project. If you add or change functionality, please update the relevant documentation in [`README.md`](../README.md) and [`REFERENCE.md`](./REFERENCE.md).

## Code of Conduct

By participating in this project, you agree to abide by the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/0/code_of_conduct/).

## Getting Help

If you need help, feel free to open an issue on our [GitHub repository](https://github.com/PRO-2684/castwright/issues) or reach out to the community.

Thank you for contributing to CastWright!
