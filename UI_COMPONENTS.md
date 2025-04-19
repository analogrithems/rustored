# Rustored UI Components Documentation

This document provides detailed information about the UI components used in the Rustored application, their layout, and how they interact with each other.

## Overview

Rustored uses the [ratatui](https://github.com/ratatui-org/ratatui) library to create a terminal-based user interface. The UI is organized according to the TDD rule #7, with a two-row layout:

- **Top Row**: Contains S3 settings, restore target selection, and specific restore target settings
- **Bottom Row**: Contains the snapshot browser and restore window (when a snapshot is selected)

## Main Layout

The main layout is defined in `src/ui/renderer.rs`. It splits the screen into two rows and then further divides the top row into three columns:

```
┌─────────────────┬─────────────────┬─────────────────┐
│                 │                 │                 │
│   S3 Settings   │ Restore Target  │ Target-Specific │
│                 │   Selection     │    Settings     │
│                 │                 │                 │
├─────────────────┴─────────────────┴─────────────────┤
│                                                     │
│                 Snapshot Browser                    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Component Details

### S3 Settings (`src/ui/components/s3_settings.rs`)

The S3 settings component displays configuration options for connecting to an S3 bucket:

- Uses a table layout with "Setting" and "Value" columns
- Includes fields for Bucket, Region, Prefix, Endpoint URL, Access Key ID, Secret Access Key, and Path Style
- Masks the Secret Access Key as `[hidden]` when not in edit mode (TDD rule #12)
- Places navigation help at the bottom (TDD rule #10)
- Highlights the currently focused field

### Restore Target Selection (`src/ui/components/restore_target.rs`)

The restore target selection component allows users to choose which type of datastore to restore to:

- Displays restore targets as a list with numeric prefixes (TDD rule #11):
  - "1. PostgreSQL"
  - "2. Elasticsearch"
  - "3. Qdrant"
- Highlights the currently selected target
- Places navigation help at the bottom (TDD rule #10)
- Shows keyboard shortcuts for target selection

### Target-Specific Settings

These components display settings specific to each restore target type:

#### PostgreSQL Settings (`src/ui/components/postgres_settings.rs`)

- Uses a table layout with "Setting" and "Value" columns
- Includes fields for Host, Port, Username, Password, Database, and SSL options
- Masks the Password as `[hidden]` when not in edit mode (TDD rule #12)
- Places navigation help at the bottom (TDD rule #10)

#### Elasticsearch Settings (`src/ui/components/elasticsearch_settings.rs`)

- Uses a table layout with "Setting" and "Value" columns
- Includes fields for Host and Index
- Places navigation help at the bottom (TDD rule #10)

#### Qdrant Settings (`src/ui/components/qdrant_settings.rs`)

- Uses a table layout with "Setting" and "Value" columns
- Includes fields for Host, Collection, and API Key
- Masks the API Key as `[hidden]` when not in edit mode (TDD rule #12)
- Places navigation help at the bottom (TDD rule #10)

### Snapshot Browser (`src/ui/components/snapshot_list.rs`)

The snapshot browser displays a list of available snapshots from the S3 bucket:

- Shows a table with columns for S3 Path, Size, and Last Modified
- Displays the full S3 path, not just the filename (TDD rule #8)
- Formats sizes and dates in human-readable format (TDD rule #9)
- Highlights the currently selected snapshot

## Navigation

The application supports various navigation methods:

- **Tab**: Cycle between different sections (S3 Settings, Restore Target Selection, Target-Specific Settings, Snapshot Browser)
- **Arrow Keys**: Navigate within a component
- **1-3 Keys**: Select different restore targets
- **Enter**: Edit the currently focused field
- **Escape**: Exit edit mode
- **t**: Test connection (consistent across all components)
- **q**: Quit the application
- **Ctrl+Z**: Suspend the application

## Styling

The UI follows consistent styling rules:

- **Labels**: Blue text
- **Focused (not editing)**: Green text with bold modifier
- **Focused (editing)**: Yellow text with bold modifier
- **Regular values**: White text
- **Headers**: Bold text
- **Borders**: Around each component
- **Help text**: Gray text at the bottom of components

## Implementation Notes

1. All components follow the TDD rules outlined in `TDD_GUIDELINES.md`
2. Navigation help text is consistently placed at the bottom of each component
3. Passwords and secrets are masked when not in edit mode
4. The layout is designed to be responsive to different terminal sizes
5. Each component logs its rendering process for debugging purposes

## Adding New Components

When adding new components to the UI:

1. Follow the existing pattern for component structure
2. Ensure compliance with all TDD rules
3. Maintain consistent styling and navigation
4. Place help text at the bottom
5. Add appropriate logging
6. Use table layouts for settings components
7. Mask any sensitive information

By following these guidelines, the UI will maintain its consistency and usability as the application evolves.
