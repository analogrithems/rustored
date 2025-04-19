# Test-Driven Development Guidelines for Rustored

This document outlines the Test-Driven Development (TDD) approach used in the Rustored project, including the rules that govern development, how to run tests, and how to fix violations. These guidelines ensure consistent quality and maintainability across the codebase.

## TDD Rules

The Rustored project follows these specific TDD rules, each with concrete implementations in the codebase:

1. **Separation of Restore Target Logic**
   - Keep all logic related to restore target types (configuring, navigating, focusing) in separate dedicated files for each supported target
   - Example: `postgres_target.rs`, `elasticsearch_target.rs`, `qdrant_target.rs`
   - **Implementation**: Each restore target has its own module with dedicated connection handling, settings components, and target-specific logic

2. **Comprehensive Comments**
   - Keep overly detailed comments about what's going on everywhere
   - Minimum 15% of lines should be comments
   - Comments should explain the "why" not just the "what"

3. **Extensive Logging**
   - Every function should log what it's doing (log at debug level for most things)
   - Log entry and exit points of functions
   - Log important state changes and decisions

4. **File Size Limits**
   - Files should not exceed 500 lines of actual code (excluding comments and blank lines) to maintain readability and maintainability
   - This encourages both modular code organization and comprehensive documentation
   - If a file grows too large, consider refactoring into smaller modules

5. **UI Navigation Tests**
   - Implement tests for UI navigation, including:
     - Tab navigation between components
     - Arrow key navigation within components
     - Popup dismissal

6. **Connection Testing**
   - Test connection key handlers for different restore targets
   - Ensure proper error handling and feedback

7. **Two-Row Layout**
   - Layout should be organized in two rows:
     - Top row: S3 settings, Restore target selection, and specific restore target settings
     - Bottom row: Snapshot browser and restore window (when a snapshot is selected)
   - **Implementation**: The `renderer.rs` file implements this layout with clear separation between top and bottom rows

8. **S3 Path Display**
   - In the snapshots table, the header should be "S3 Path" and show the full path from S3, not just the filename
   - **Implementation**: The `snapshot_list.rs` component displays the full S3 path in the table

9. **Human-Readable Formats**
   - Size and Last Modified columns in the snapshots table should always be displayed in human-readable format
   - Sizes should be in MB with appropriate decimal places
   - Dates should be in a clear, consistent format
   - **Implementation**: The `snapshot_list.rs` component formats sizes as "XX.XX MB" and dates as "YYYY-MM-DD HH:MM:SS"

10. **Navigation Help Placement**
    - Navigation help text should be placed at the bottom of any window that has it
    - Help text should use consistent formatting across all components
    - **Implementation**: All components (S3 settings, PostgreSQL settings, Elasticsearch settings, Qdrant settings, and restore target selection) place navigation help at the bottom with consistent "↑↓ Navigate" format

11. **Numeric Restore Target Listing**
    - Restore targets in the restore target selection window should be listed numerically
    - Example: "1. PostgreSQL", "2. Elasticsearch", "3. Qdrant"
    - **Implementation**: The `restore_target.rs` component displays targets as a list with numeric prefixes

12. **Password and Secret Masking**
    - Passwords or secrets should be masked when not in edit mode
    - Use "[hidden]" to mask sensitive information consistently across components
    - Only show actual values when the user is actively editing the field
    - **Implementation**: S3 Secret Access Key, PostgreSQL password, and Qdrant API key all use "[hidden]" masking when not in edit mode

## Running TDD Tests

To ensure your code adheres to the TDD rules, follow these steps:

### 1. Automated Tests

Run the automated test suite to verify UI navigation, connection handling, and other testable rules:

```bash
cargo test
```

### 2. Manual Verification

Some rules require manual verification:

```bash
# Run the application
cargo run -- browse-snapshots

# Check for rule compliance:
# - Verify two-row layout
# - Check navigation help placement
# - Confirm numeric restore target listing
# - Verify human-readable formats
# - Check full S3 paths
```

### 3. Code Analysis

For rules related to code structure and quality:

```bash
# Check file sizes
find src -name "*.rs" -exec wc -l {} \; | sort -nr

# Count comments vs. code lines (requires cloc tool)
cloc src/ --by-file

# Check logging coverage
grep -r "debug!" src/ --include="*.rs" | wc -l
```

## Fixing TDD Rule Violations

When you encounter violations of the TDD rules, use these prompts to fix them:

### Rule 1: Separation of Restore Target Logic

```markdown
Move the [specific functionality] from [source file] to the appropriate target-specific file [target file].
```

### Rule 2: Comprehensive Comments

```markdown
Increase comment coverage in [file] by adding detailed explanations for [specific functions/sections].
```

### Rule 3: Extensive Logging

```markdown
Add debug logging statements to [function] to track its execution flow and important state changes.
```

### Rule 4: File Size Limits

```markdown
Refactor [large file] by extracting [specific functionality] into a new module to reduce its size below 500 lines.
```

### Rule 5 & 6: UI Navigation and Connection Tests

```markdown
Implement tests for [specific navigation pattern/connection handler] in [file] to ensure proper functionality.
```

### Rule 7: Two-Row Layout

```markdown
Adjust the layout in renderer.rs to ensure [component] is positioned in the [top/bottom] row according to the two-row layout rule.
```

### Rule 8: S3 Path Display

```markdown
Update the snapshots table to display the full S3 path instead of just the filename and change the header to 'S3 Path'.
```

### Rule 9: Human-Readable Formats

```markdown
Format the [size/date] column in the snapshots table to use human-readable format by [specific formatting approach].
```

### Rule 10: Navigation Help Placement

```markdown
Move the navigation help text in [component] from [current position] to the bottom of the window as required by TDD rule #10.
```

### Rule 11: Numeric Restore Target Listing

```markdown
Update the restore target selection to list targets numerically (e.g., "1. PostgreSQL", "2. Elasticsearch", etc.) instead of [current format].
```

### Rule 12: Password and Secret Masking

```markdown
Implement masking for the [password/secret field] in [component] to hide the actual value when not in edit mode.
```

## Best Practices

1. **Always Check Before Committing**
   - Run the verification steps before committing changes
   - Address all rule violations before considering work complete

2. **Incremental Compliance**
   - When working on a new feature, ensure it complies with all rules from the start
   - When fixing an existing feature, bring it into compliance with all rules

3. **Documentation Updates**
   - If you modify or extend the TDD rules, update this document
   - Document any exceptions or special cases

4. **Rule Priorities**
   - If rules conflict, prioritize in this order:
     1. Functionality (the code must work correctly)
     2. Maintainability (comments, file size, separation of concerns)
     3. UI consistency (layout, formatting, help text)

By following these guidelines, we ensure that the Rustored project maintains high quality, consistency, and maintainability throughout its development lifecycle.
