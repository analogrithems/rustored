<!-- markdownlint-disable MD034 -->

# Rustored v0.2.0 (2025-04-19)

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Rustored Logo](logo.png)
[*] — Dont panic, your data is safe!

Rustored is a terminal-based CLI and TUI application for downloading and restoring snapshots from AWS S3 to various datastores including Postgres, Elasticsearch, and Qdrant. Built with a Test-Driven Development (TDD) approach, it ensures high quality and maintainability.

## Features

- Browse and select snapshots stored in S3 via an interactive TUI
- Download snapshots with progress feedback
- Restore snapshots to Postgres, Elasticsearch, or Qdrant
- Flexible configuration via CLI flags or environment variables
- Intuitive navigation with keyboard shortcuts
- Dynamic UI that adapts to the selected restore target
- Ability to suspend the application with Ctrl+Z
- Consistent table-based layout for all settings components
- Secure password and API key masking
- Human-readable formats for file sizes and timestamps
- Numerically listed restore targets for easy selection

## Installation

### Download a Binary Release

Prebuilt binaries for Linux, macOS, and Windows are available on the [GitHub Releases page](https://github.com/${{ github.repository_owner }}/rustored/releases). Download the latest release for your platform and make it executable if needed:

```bash
# Example for Linux:
wget <https://github.com/${{ github.repository_owner }}/rustored/releases/latest/download/rustored>
chmod +x rustored
./rustored --help
```

### Run with Docker

You can run rustored directly from a secure, minimal container image:

```bash
docker run --rm -it \
  ghcr.io/${{ github.repository_owner }}/rustored:latest \
  --s3-bucket <BUCKET> --s3-access-key-id <KEY> --s3-secret-access-key <SECRET> \
  --ds-type <postgres|elasticsearch|qdrant> [datastore options...]
```

Or launch the TUI interactively:

```bash
docker run --rm -it \
  --env TERM=xterm-256color \
  --env <YOUR_ENV_VARS> \
  ghcr.io/${{ github.repository_owner }}/rustored:latest
```

### Build from Source

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
rustored browse-snapshots
```

**Navigation:**

- Use arrow keys or j/k to navigate the snapshot list
- Press Tab to cycle between different sections (S3 Settings, Restore Target, Snapshot List)
- Press 1, 2, or 3 to select different restore targets (PostgreSQL, Elasticsearch, Qdrant)
- Press e to edit the currently focused field
- Press Enter to confirm selection or save edits
- Press q to quit the application
- Press Ctrl+Z to suspend the application

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

MIT Your Name
