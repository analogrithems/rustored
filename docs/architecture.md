# Rustored Architecture

## Overview

Rustored is a terminal-based application for browsing and restoring snapshots from S3 to various datastores. The application is built with a modular architecture that separates concerns and makes the code more maintainable.

## Core Components

### SnapshotBrowser

The `SnapshotBrowser` is responsible for:

- Browsing and selecting snapshots from S3
- Managing UI state (focus, input mode, popup state)
- Delegating restore operations to the `RestoreBrowser`

### RestoreBrowser

The `RestoreBrowser` is a new component introduced in v0.2.0 that handles all restore logic:

- Takes configuration for different restore targets (Postgres, Elasticsearch, Qdrant)
- Validates configurations before attempting restore
- Performs the actual restore operation
- Reports progress and errors back to the user

This separation of concerns improves code organization and makes it easier to extend the restore functionality in the future.

### Configuration Models

Each supported datastore has its own configuration model:

- `PostgresConfig`: Configuration for PostgreSQL connections
- `ElasticsearchConfig`: Configuration for Elasticsearch connections
- `QdrantConfig`: Configuration for Qdrant connections
- `S3Config`: Configuration for S3 connections

These models are created only when needed based on the selected restore target, which improves resource usage.

### UI Rendering

The UI is rendered using the Ratatu TUI library, with separate rendering functions for:

- Main application UI (`ui` function)
- Restore UI (`restore_ui` function)

## Data Flow

1. User selects a snapshot in the `SnapshotBrowser`
2. User confirms restore operation
3. `SnapshotBrowser` delegates to `RestoreBrowser` for the actual restore
4. `RestoreBrowser` validates configuration and performs restore
5. Results are displayed to the user

## Configuration

Configuration can be provided via:

- Command-line arguments
- Environment variables
- Interactive input in the TUI

Each configuration model now has a `new()` method that loads default values from environment variables, making it easier to configure the application.

## Error Handling

Errors are handled using the `anyhow` crate and displayed to the user via popup messages in the TUI or in the `rustored.log`
