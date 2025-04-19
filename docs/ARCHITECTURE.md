# Rustored Architecture

This document provides an overview of the Rustored application architecture.

## Application Structure

Rustored follows a modular architecture with clear separation of concerns:

```
rustored/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports
│   ├── postgres.rs             # PostgreSQL connection and operations
│   ├── restore.rs              # Restore interface and implementations
│   ├── targets/                # Target-specific implementations
│   │   ├── elasticsearch_target.rs
│   │   ├── postgres_target.rs
│   │   └── qdrant_target.rs
│   └── ui/                     # User interface components
│       ├── app.rs              # Application state
│       ├── browser.rs          # S3 snapshot browser
│       ├── components.rs       # Reusable UI components
│       ├── key_handler.rs      # Keyboard input handling
│       ├── layouts.rs          # UI layout definitions
│       ├── models.rs           # Data models for UI state
│       ├── renderer.rs         # Terminal rendering
│       └── rustored.rs         # Main UI application logic
└── tests/                      # Integration tests
```

## Core Components

### UI Layer

The UI layer is built using the [ratatui](https://github.com/ratatui-org/ratatui) library and follows the Model-View-Controller pattern:

- **Models** (`models.rs`): Data structures representing application state
- **Views** (`renderer.rs`, `layouts.rs`): Terminal rendering and layout
- **Controllers** (`key_handler.rs`, `rustored.rs`): Input handling and state updates

### Data Layer

- **S3 Browser** (`browser.rs`): Handles browsing and downloading snapshots from S3
- **Restore Targets** (`targets/`): Implementations for different database restore targets

### Core Logic

- **Restore Interface** (`restore.rs`): Defines the common interface for all restore targets
- **PostgreSQL Operations** (`postgres.rs`): PostgreSQL-specific operations

## Data Flow

1. User interacts with the terminal UI
2. Key events are processed by `key_handler.rs`
3. Application state is updated in `rustored.rs`
4. S3 operations are performed by `browser.rs`
5. Restore operations are delegated to the appropriate target implementation
6. UI is refreshed to reflect the updated state

## Design Principles

Rustored follows these key design principles:

1. **Modularity**: Clear separation of concerns with focused modules
2. **Testability**: Components designed for easy testing
3. **Error Handling**: Comprehensive error handling with the `anyhow` crate
4. **Logging**: Detailed logging throughout the application
5. **Documentation**: Well-documented code and user interface

This architecture allows for easy extension with new restore targets and UI features while maintaining a clean, maintainable codebase.
