# Rustored UI Navigation Guide

This document provides a comprehensive guide to the navigation system in the Rustored terminal user interface (TUI).

## UI Layout

The Rustored UI is organized according to TDD rule #7, with a two-row layout:

```ascii
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

1. **Top Row**: Contains three panels side by side:
   - **S3 Settings** (left panel): Configuration for S3 connection parameters
   - **Restore Target Selection** (middle panel): Options to select the target datastore type (PostgreSQL, Elasticsearch, Qdrant)
   - **Target-Specific Settings** (right panel): Dynamic settings panel that changes based on the selected restore target

2. **Bottom Row**: Contains the snapshot browser that displays available snapshots from S3

For a more detailed view of the UI layout, see the [full diagram](../images/ui_layout.svg).

## Keyboard Navigation

### Window Navigation

| Key | Action |
|-----|--------|
| `Tab` | Cycle between main window sections (S3 Settings → Restore Target Settings → Snapshot List → S3 Settings) |
| `q` | Quit the application |
| `Ctrl+Z` | Suspend the application (Unix systems only) |
| `r` | Reload snapshots from S3 |
| `t` | When focus is on S3 Settings: Test S3 connection |

### Restore Target Selection

| Key | Action |
|-----|--------|
| `1` | Select PostgreSQL as the restore target |
| `2` | Select Elasticsearch as the restore target |
| `3` | Select Qdrant as the restore target |

When you select a different restore target, the Restore Settings panel will automatically update to show the appropriate settings for that target. Additionally, the focus will move to the first field in the selected target's settings if it wasn't already on a field for that target.

### Within-Window Navigation

| Key | Action |
|-----|--------|
| `↓` or `j` | Move to the next field within the current window |
| `↑` or `k` | Move to the previous field within the current window |
| `Enter` | When in Snapshot List: Select the highlighted snapshot for restore |

The up/down navigation wraps around within each window section. For example, if you're on the last field in the S3 Settings window and press Down, you'll cycle back to the first field in that window.

### Field Editing

| Key | Action |
|-----|--------|
| `Enter` | When on a field: Enter edit mode for the currently focused field |
| `Enter` | When already in edit mode: Save the edited value |
| `Esc` | Cancel editing |

## Focus Indicators

The currently focused field or section is highlighted in yellow. This provides visual feedback about which element will receive keyboard input.

## Popup Windows

Popup windows appear in certain situations:

- **Confirmation**: When confirming an action like restoring a snapshot
- **Progress**: When downloading or restoring a snapshot
- **Error/Success**: When an operation completes or fails

Press `Esc` or `Enter` to dismiss most popups.

## Example Workflow

1. Use `Tab` to navigate to the S3 Settings window
2. Use `↓`/`↑` or `j`/`k` to navigate between fields within the S3 Settings window
3. Press `Enter` to edit the currently focused field
4. Press `t` to test the S3 connection
5. Press `r` to load snapshots from S3
6. Press `Tab` to navigate to the Restore Target Settings window
7. Press `1`, `2`, or `3` to select the desired restore target
8. Use `↓`/`↑` or `j`/`k` to navigate between fields within the Restore Target Settings window
9. Press `Tab` to navigate to the Snapshot List window
10. Use `↓`/`↑` or `j`/`k` to select a snapshot
11. Press `Enter` to confirm and start the restore process

## Implementation Details

The navigation system is implemented in the `RustoredApp.handle_key_event` method, which processes all key events and updates the application state accordingly. The UI rendering is handled by the `renderer.rs` file, which uses the Ratatui library to create the layout and render the components.

When switching between restore targets, the focus is automatically updated to ensure a smooth user experience. This is achieved through the following code in the key event handler:

```rust
// Tab key handling - only changes window focus
KeyCode::Tab => {
    // Cycle between main window sections only
    self.focus = match self.focus {
        // S3 Settings fields - move to Restore Target settings
        FocusField::Bucket | FocusField::Region | /* other S3 fields */ => {
            // Move to restore target settings
            match self.restore_target {
                RestoreTarget::Postgres => FocusField::PgHost,
                RestoreTarget::Elasticsearch => FocusField::EsHost,
                RestoreTarget::Qdrant => FocusField::EsHost,
            }
        }
        // Restore Target settings - move to Snapshot List
        FocusField::PgHost | /* other Postgres fields */ |
        FocusField::EsHost | /* other ES fields */ => FocusField::SnapshotList,
        // Snapshot list - move back to S3 Settings
        FocusField::SnapshotList => FocusField::Bucket,
        // Default case
        _ => FocusField::Bucket,
    };
}

// Down key handling - navigates within a window
KeyCode::Down | KeyCode::Char('j') => {
    if self.focus == FocusField::SnapshotList {
        // Navigate snapshot list
        self.snapshot_browser.selected_index = 
            (self.snapshot_browser.selected_index + 1) % self.snapshot_browser.snapshots.len();
    } else {
        // Navigate within settings windows
        self.focus = match self.focus {
            // S3 Settings navigation
            FocusField::Bucket => FocusField::Region,
            // ... other field navigation logic
        };
    }
}
