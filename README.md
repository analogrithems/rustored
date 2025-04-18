# Rustored

![Rustored Logo](logo.png)
```[✓] Dont panic - your data is safe.```

Rustored is a terminal-based CLI and TUI application for downloading and restoring snapshots from AWS S3 to various datastores including Postgres, Elasticsearch, and Qdrant.

## Features

- Browse and select snapshots stored in S3 via an interactive TUI
- Download snapshots with progress feedback
- Restore snapshots to Postgres, Elasticsearch, or Qdrant
- Flexible configuration via CLI flags or environment variables

## Installation

```bash
cargo install --path .
```

## Usage

### CLI Mode

Download and restore via direct commands:

```bash
rustored --s3-bucket <BUCKET> --s3-access-key-id <KEY> --s3-secret-access-key <SECRET> \
         --ds-type <postgres|elasticsearch|qdrant> [datastore options...]
```

### TUI Mode

Simply run without subcommands to launch the interactive UI:

```bash
rustored
```

Use arrow keys to navigate, Enter to confirm download, and restore.

## Configuration

All settings can be provided via `--flag` or corresponding environment variables:

| Flag                              | Env Var                   | Description                          |
| --------------------------------- | ------------------------- | ------------------------------------ |
| `--s3-bucket`                     | `S3_BUCKET`               | S3 bucket name                       |
| `--s3-prefix`                     | `S3_PREFIX`               | (Optional) S3 key prefix             |
| `--s3-region`                     | `S3_REGION`               | (Optional) AWS region                |
| `--s3-access-key-id`              | `S3_ACCESS_KEY_ID`        | AWS access key ID                    |
| `--s3-secret-access-key`          | `S3_SECRET_ACCESS_KEY`    | AWS secret access key                |
| `--ds-type`                       | `DS_TYPE`                 | Datastore type: postgres, elasticsearch, qdrant |
| `--ds-postgres-conn`              | `DS_POSTGRES_CONN`        | Postgres connection string           |
| `--ds-es-url`                     | `DS_ES_URL`               | Elasticsearch URL                    |
| `--ds-es-user`                    | `DS_ES_USER`              | Elasticsearch username               |
| `--ds-es-pass`                    | `DS_ES_PASS`              | Elasticsearch password               |
| `--ds-qdrant-url`                 | `DS_QDRANT_URL`           | Qdrant API URL                       |
| `--ds-qdrant-api`                 | `DS_QDRANT_API`           | (Optional) Qdrant API key            |

## Contributing

Contributions welcome! Each datastore restore implementation lives in its own module under `src/restore/`.

## License

MIT © Your Name
