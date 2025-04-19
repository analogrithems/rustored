# Rustored Documentation

Welcome to the Rustored documentation. This guide provides information about the Rustored application, a tool for managing and restoring database backups from S3.

## Overview

Rustored is a terminal-based application built with Rust that allows you to:
- Browse S3 buckets for database backups
- Restore backups to various database targets (PostgreSQL, Elasticsearch, Qdrant)
- Configure connection settings for S3 and database targets

## Documentation Sections

### User Interface
- [UI Components](UI_COMPONENTS.md) - Overview of the UI components
- [UI Navigation](ui/navigation.md) - Guide to navigating the application

### Technical Documentation
- [TDD Guidelines](TDD_GUIDELINES.md) - Test-Driven Development guidelines for the project
- [Architecture](ARCHITECTURE.md) - Overview of the application architecture

### Target-Specific Documentation
- [PostgreSQL](targets/postgres.md) - PostgreSQL-specific documentation
- [Elasticsearch](targets/elasticsearch.md) - Elasticsearch-specific documentation
- [Qdrant](targets/qdrant.md) - Qdrant-specific documentation

## Getting Started

To get started with Rustored, clone the repository and build the application:

```bash
git clone https://github.com/analogrithems/rustored.git
cd rustored
cargo build --release
```

Run the application:

```bash
./target/release/rustored
```

## Contributing

Contributions are welcome! Please read the [contributing guidelines](CONTRIBUTING.md) before submitting a pull request.
