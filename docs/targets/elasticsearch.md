# Elasticsearch Target Documentation

This document provides information about using Elasticsearch as a restore target in Rustored.

## Configuration

The Elasticsearch target requires the following configuration parameters:

| Parameter | Description | Example |
|-----------|-------------|---------|
| Host | Elasticsearch server hostname | `localhost` |
| Port | Elasticsearch server port | `9200` |
| Username | Elasticsearch username (optional) | `elastic` |
| Password | Elasticsearch password (optional) | `******` |
| Index | Target index name | `myindex` |

## Supported Backup Formats

The Elasticsearch target supports restoring from the following backup formats:

- Elasticsearch snapshot files (`.es`)
- JSON document collections (`.json`)

## Restore Process

When restoring to an Elasticsearch target, Rustored performs the following steps:

1. Downloads the selected snapshot from S3
2. Validates the backup file format
3. Establishes a connection to the Elasticsearch server
4. Creates the target index if it doesn't exist
5. Executes the restore operation by indexing the documents
6. Reports progress during the restore operation
7. Verifies the restore completed successfully

## Example Usage

1. Select Elasticsearch as the restore target by pressing `2` in the Restore Target panel
2. Configure the Elasticsearch connection parameters
3. Test the connection by pressing `t` with focus on the Elasticsearch settings panel
4. Browse and select a snapshot from the Snapshot Browser
5. Press `Enter` to initiate the restore process
6. Confirm the restore operation in the popup dialog

## Troubleshooting

### Connection Issues

If you encounter connection issues:

- Verify the host and port are correct
- Ensure the Elasticsearch server is running and accessible
- Check that the provided username and password are valid
- Confirm that the server is accepting HTTP connections

### Restore Failures

If the restore operation fails:

- Check the Elasticsearch server logs for detailed error messages
- Verify that the user has sufficient privileges to create and write to indices
- Ensure there is enough disk space available
- Check that the backup file is not corrupted
- Verify that the index mapping is compatible with the data being restored
