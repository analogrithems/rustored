# PostgreSQL Target Documentation

This document provides information about using PostgreSQL as a restore target in Rustored.

## Configuration

The PostgreSQL target requires the following configuration parameters:

| Parameter | Description | Example |
|-----------|-------------|---------|
| Host | PostgreSQL server hostname | `localhost` |
| Port | PostgreSQL server port | `5432` |
| Username | PostgreSQL username | `postgres` |
| Password | PostgreSQL password | `******` |
| Database | Target database name | `mydb` |

## Supported Backup Formats

The PostgreSQL target supports restoring from the following backup formats:

- PostgreSQL custom format dumps (`.dump`)
- Plain SQL dumps (`.sql`)

## Restore Process

When restoring to a PostgreSQL target, Rustored performs the following steps:

1. Downloads the selected snapshot from S3
2. Validates the backup file format
3. Establishes a connection to the PostgreSQL server
4. Executes the restore operation using the appropriate method based on the file format
5. Reports progress during the restore operation
6. Verifies the restore completed successfully

## Example Usage

1. Select PostgreSQL as the restore target by pressing `1` in the Restore Target panel
2. Configure the PostgreSQL connection parameters
3. Test the connection by pressing `t` with focus on the PostgreSQL settings panel
4. Browse and select a snapshot from the Snapshot Browser
5. Press `Enter` to initiate the restore process
6. Confirm the restore operation in the popup dialog

## Troubleshooting

### Connection Issues

If you encounter connection issues:

- Verify the host and port are correct
- Ensure the PostgreSQL server is running and accessible
- Check that the provided username and password are valid
- Confirm that the specified database exists

### Restore Failures

If the restore operation fails:

- Check the PostgreSQL server logs for detailed error messages
- Verify that the user has sufficient privileges to restore the database
- Ensure there is enough disk space available
- Check that the backup file is not corrupted
