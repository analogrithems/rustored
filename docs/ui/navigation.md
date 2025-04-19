# Rustored UI Navigation Guide

This document provides a comprehensive guide to the navigation system in the Rustored terminal user interface (TUI).

## UI Layout

The Rustored UI is organized into three main sections:

![UI Layout Diagram](../images/ui_layout.png)

1. **Title Bar**: Displays the application title at the top of the screen
2. **Settings Row**: Contains three panels side by side:
   - **S3 Settings**: Configuration for S3 connection (left panel)
   - **Restore Target**: Selection options for the target type (middle panel)
   - **Restore Settings**: Dynamic settings panel for the selected target (right panel)
3. **Snapshot List**: Displays available snapshots from S3 (bottom panel)

## Keyboard Navigation

### Global Navigation

| Key | Action |
|-----|--------|
| `Tab` | Cycle between different sections (S3 Settings, Restore Target, Snapshot List) |
| `q` | Quit the application |
| `Ctrl+Z` | Suspend the application (Unix systems only) |
| `r` | Reload snapshots from S3 |

### Restore Target Selection

| Key | Action |
|-----|--------|
| `1` | Select PostgreSQL as the restore target |
| `2` | Select Elasticsearch as the restore target |
| `3` | Select Qdrant as the restore target |

When you select a different restore target, the Restore Settings panel will automatically update to show the appropriate settings for that target. Additionally, the focus will move to the first field in the selected target's settings if it wasn't already on a field for that target.

### Snapshot List Navigation

| Key | Action |
|-----|--------|
| `↓` or `j` | Move down in the snapshot list |
| `↑` or `k` | Move up in the snapshot list |
| `Enter` | Select the highlighted snapshot for restore |

### Field Editing

| Key | Action |
|-----|--------|
| `e` | Edit the currently focused field |
| `Enter` | Save the edited value |
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

1. Use `Tab` to navigate to the S3 Settings section
2. Press `e` to edit fields like bucket name, region, etc.
3. Press `r` to load snapshots from S3
4. Use `Tab` to navigate to the Restore Target section
5. Press `1`, `2`, or `3` to select the desired restore target
6. Configure the target-specific settings in the Restore Settings panel
7. Use `Tab` to navigate to the Snapshot List
8. Use `↓`/`↑` or `j`/`k` to select a snapshot
9. Press `Enter` to confirm and start the restore process

## Implementation Details

The navigation system is implemented in the `RustoredApp.handle_key_event` method, which processes all key events and updates the application state accordingly. The UI rendering is handled by the `renderer.rs` file, which uses the Ratatui library to create the layout and render the components.

When switching between restore targets, the focus is automatically updated to ensure a smooth user experience. This is achieved through the following code in the key event handler:

```rust
KeyCode::Char('1') => {
    self.restore_target = RestoreTarget::Postgres;
    // Set focus to first PostgreSQL field if not already on a PostgreSQL field
    if !matches!(self.focus, 
        FocusField::PgHost | 
        FocusField::PgPort | 
        FocusField::PgUsername | 
        FocusField::PgPassword | 
        FocusField::PgSsl | 
        FocusField::PgDbName
    ) {
        self.focus = FocusField::PgHost;
    }
}
```
