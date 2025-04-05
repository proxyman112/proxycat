# Contributing to ProxyCat

Thank you for your interest in contributing to ProxyCat! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please read it before contributing.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check the issue list as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* Use a clear and descriptive title
* Describe the exact steps which reproduce the problem
* Provide specific examples to demonstrate the steps
* Describe the behavior you observed after following the steps
* Explain which behavior you expected to see instead and why
* Include screenshots and animated GIFs if possible
* Include the exact error messages and stack traces if applicable

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* A clear and descriptive title
* A detailed description of the proposed functionality
* Explain why this enhancement would be useful
* List any alternative solutions or features you've considered

### Pull Requests

* Fill in the required template
* Do not include issue numbers in the PR title
* Include screenshots and animated GIFs in your pull request whenever possible
* Follow the Rust styleguides
* Include thoughtfully-worded, well-structured tests
* Document new code
* End all files with a newline

## Development Setup

1. Fork and clone the repository
2. Install Rust 1.75 or later
3. Install development dependencies:
   ```bash
   cargo install cargo-fmt cargo-clippy
   ```
4. Set up your development environment:
   ```bash
   cargo build
   ```

## Code Style

* Use `cargo fmt` to format your code
* Use `cargo clippy` to check for linting issues
* Follow the [Rust Style Guide](https://rust-lang.github.io/api-guidelines/)
* Use meaningful variable and function names
* Add comments for complex logic
* Keep functions focused and small

## Testing

* Write unit tests for new functionality
* Ensure all tests pass before submitting a PR
* Add integration tests for new features
* Test on Windows 7 or later

## Documentation

* Update the README.md if needed
* Add documentation for new features
* Include examples for new functionality
* Update the CHANGELOG.md for significant changes

## Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
* Consider starting the commit message with an applicable emoji:
    * 🎨 `:art:` when improving the format/structure of the code
    * 🐎 `:racehorse:` when improving performance
    * 🚱 `:non-potable_water:` when plugging memory leaks
    * 📝 `:memo:` when writing docs
    * 🐛 `:bug:` when fixing a bug
    * 🔥 `:fire:` when removing code or files
    * 💚 `:green_heart:` when fixing the CI build
    * ✅ `:white_check_mark:` when adding tests
    * 🔒 `:lock:` when dealing with security
    * ⬆️ `:arrow_up:` when upgrading dependencies
    * ⬇️ `:arrow_down:` when downgrading dependencies

## Questions?

Feel free to open an issue for any questions or concerns you might have about contributing to ProxyCat. 