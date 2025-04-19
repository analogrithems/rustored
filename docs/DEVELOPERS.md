# Rustored Developer Documentation

This document provides technical details for developers working on the Rustored codebase.

## Architecture

Rustored is built using a modular architecture with the following main components:

- **CLI Interface**: Handles command-line arguments and environment variables
- **TUI Interface**: Provides an interactive terminal user interface
- **S3 Client**: Manages connections to AWS S3 for listing and downloading snapshots
- **Restore Modules**: Implementations for each supported datastore (Postgres, Elasticsearch, Qdrant)

## Navigation System

The navigation system has been refactored to provide a more intuitive user experience:

### Key Components

- **RustoredApp**: The main application struct that handles global state and navigation
- **SnapshotBrowser**: Manages the snapshot listing and selection
- **Renderer**: Handles the UI layout and rendering

### Navigation Flow

1. The `RustoredApp.handle_key_event` method processes all key events
2. Main navigation keys:
   - **Tab**: Cycles between main window sections (S3 Settings → Restore Target Settings → Snapshot List → S3 Settings)
   - **Up/Down Arrows** or **j/k**: Navigate within the current window section
   - **1, 2, 3**: Select different restore targets (PostgreSQL, Elasticsearch, Qdrant)
   - **q**: Quits the application
   - **Ctrl+Z**: Suspends the application (Unix systems only)

### Focus Management

When switching between restore targets, the focus automatically moves to the appropriate field if it wasn't already on a field for that target. This is handled in the key event processing for the '1', '2', and '3' keys.

## UI Layout

The UI is organized into three main sections:

1. **Title Bar**: Displays the application title
2. **Settings Row**: Contains three panels side by side:
   - **S3 Settings**: Configuration for S3 connection
   - **Restore Target**: Selection options for the target type
   - **Restore Settings**: Dynamic settings panel for the selected target
3. **Snapshot List**: Displays available snapshots from S3

### Layout Implementation

The layout is implemented using Ratatui's layout system:

```rust
// Main vertical layout
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Title
        Constraint::Length(8),  // S3 Settings & Restore Target (horizontal row)
        Constraint::Min(10),    // Snapshot List
    ])
    .split(f.size());

// Horizontal layout for settings row
let horizontal_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(30),  // S3 Settings
        Constraint::Percentage(20),  // Restore Target
        Constraint::Percentage(50),  // Restore Settings
    ])
    .split(chunks[1]);
```

## Adding New Features

When adding new features, consider the following guidelines:

1. **Navigation**: Add new key handlers to the `RustoredApp.handle_key_event` method
2. **UI Components**: Create new rendering functions in renderer.rs
3. **Models**: Add new data structures to the models module
4. **Tests**: Update or add tests in the tests directory

## Testing

The codebase includes both unit tests and snapshot tests:

- **Unit Tests**: Test individual components and functions
- **Snapshot Tests**: Verify UI rendering and layout

Run tests with:

```bash
cargo test
```

## Debugging

For debugging the TUI, you can use the following techniques:

1. **Logging**: Use the `debug!` macro to log to a file
2. **Popup Messages**: Display debug information in popup windows
3. **Environment Variables**: Set `RUST_LOG=debug` for verbose logging
