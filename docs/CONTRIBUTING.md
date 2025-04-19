# Contributing to Rustored

Thank you for your interest in contributing to Rustored! This document provides guidelines and instructions for contributing to the project.

## TDD Guidelines

Rustored follows Test-Driven Development (TDD) principles. Please review the [TDD Guidelines](TDD_GUIDELINES.md) before contributing.

## Development Workflow

1. **Fork the repository** on GitHub
2. **Clone your fork** to your local machine
3. **Create a new branch** for your feature or bugfix
4. **Make your changes** following the coding standards
5. **Add tests** for your changes
6. **Ensure all tests pass** by running `cargo test`
7. **Commit your changes** with clear, descriptive commit messages
8. **Push your branch** to your fork on GitHub
9. **Submit a pull request** to the main repository

## Coding Standards

- Follow Rust's official [style guidelines](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use `rustfmt` to format your code
- Run `clippy` to catch common mistakes and improve your code
- Ensure your code has no warnings
- Add comments to your code (minimum 15% as per TDD guidelines)
- Keep file sizes under 500 lines (excluding comments)
- Add appropriate logging statements to all functions

## Pull Request Process

1. Ensure your code follows the coding standards
2. Update the documentation if necessary
3. Add or update tests as appropriate
4. Make sure all tests pass
5. The PR will be reviewed by maintainers
6. Address any feedback from the review
7. Once approved, your PR will be merged

## Reporting Bugs

When reporting bugs, please include:

- A clear, descriptive title
- Steps to reproduce the bug
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Your environment (OS, Rust version, etc.)

## Feature Requests

Feature requests are welcome! Please provide:

- A clear, descriptive title
- A detailed description of the proposed feature
- Any relevant examples or mockups
- Why this feature would be useful to the project

## Code of Conduct

- Be respectful and inclusive
- Focus on the issue, not the person
- Be patient and constructive
- Help others when you can

Thank you for contributing to Rustored!
