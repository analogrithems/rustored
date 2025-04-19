# Qdrant Target Documentation

This document provides information about using Qdrant as a restore target in Rustored.

## Configuration

The Qdrant target requires the following configuration parameters:

| Parameter | Description | Example |
|-----------|-------------|---------|
| Host | Qdrant server hostname | `localhost` |
| Port | Qdrant server port | `6333` |
| API Key | Qdrant API key (optional) | `******` |
| Collection | Target collection name | `mycollection` |

## Supported Backup Formats

The Qdrant target supports restoring from the following backup formats:

- Qdrant snapshot files (`.qdrant`)
- JSON vector collections (`.json`)

## Restore Process

When restoring to a Qdrant target, Rustored performs the following steps:

1. Downloads the selected snapshot from S3
2. Validates the backup file format
3. Establishes a connection to the Qdrant server
4. Creates the target collection if it doesn't exist
5. Executes the restore operation by uploading the vectors
6. Reports progress during the restore operation
7. Verifies the restore completed successfully

## Example Usage

1. Select Qdrant as the restore target by pressing `3` in the Restore Target panel
2. Configure the Qdrant connection parameters
3. Test the connection by pressing `t` with focus on the Qdrant settings panel
4. Browse and select a snapshot from the Snapshot Browser
5. Press `Enter` to initiate the restore process
6. Confirm the restore operation in the popup dialog

## Troubleshooting

### Connection Issues

If you encounter connection issues:

- Verify the host and port are correct
- Ensure the Qdrant server is running and accessible
- Check that the provided API key is valid
- Confirm that the server is accepting HTTP/gRPC connections

### Restore Failures

If the restore operation fails:

- Check the Qdrant server logs for detailed error messages
- Verify that the user has sufficient privileges to create and write to collections
- Ensure there is enough disk space available
- Check that the backup file is not corrupted
- Verify that the collection schema is compatible with the data being restored
