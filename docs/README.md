# Rustored Documentation Index

This index provides an overview of all documentation available for the Rustored project, a PostgreSQL database management tool with support for multiple datastores.

## Core Documentation

| Document | Description |
|----------|-------------|
| [TDD Guidelines](./TDD_GUIDELINES.md) | Outlines the Test-Driven Development approach used in Rustored, including the 12 specific rules that govern development, testing procedures, and how to fix violations. Essential reading for all contributors. |
| [Developers Guide](./DEVELOPERS.md) | Technical details for developers working on the Rustored codebase, including architecture overview, component descriptions, and development workflows. |
| [CI/CD & Release Process](./CICD.md) | Describes how GitHub Actions workflows are structured for Rustored and how to trigger builds, tests, Docker image pushes, and new binary releases. |

## User Interface Documentation

| Document | Description |
|----------|-------------|
| [UI Components](./UI_COMPONENTS.md) | Detailed information about the UI components used in Rustored, their layout, and how they interact with each other. Includes diagrams of the two-row layout and descriptions of each component. |
| [UI Navigation](./ui/navigation.md) | Comprehensive guide to the navigation system in the Rustored terminal user interface, including keyboard shortcuts, focus management, and interaction patterns. |

## Getting Started

For new users and contributors, we recommend reading the documentation in this order:

1. **TDD Guidelines** - To understand the development principles
2. **Developers Guide** - For an overview of the codebase architecture
3. **UI Components** - To learn about the user interface structure
4. **UI Navigation** - To understand how users interact with the application
5. **CI/CD & Release Process** - For contributors who need to work with the build pipeline

## Contributing to Documentation

When adding new documentation:

1. Place the documentation file in the appropriate directory
2. Use Markdown format with proper headings and formatting
3. Add a link to the new document in this index
4. Provide a brief but informative description
5. Follow the TDD guidelines for documentation (minimum 15% comments)

## Additional Resources

- [GitHub Repository](https://github.com/analogrithems/rustored)
- [Issue Tracker](https://github.com/analogrithems/rustored/issues)
- [Release Notes](https://github.com/analogrithems/rustored/releases)
