# Flux Backend

## Overview

 Rust-based mutation testing backend. It provides APIs for managing mutation tests, running mutation analysis, and reporting results. The backend is built with Axum, SQLx, and supports observability with tracing, Prometheus, and Jaeger.

## Features
- Create, list, and manage mutation tests
- Run mutation analysis on source code
- RESTful API endpoints
- Observability: structured logging, metrics, tracing
- Configurable via environment variables and config files

## Getting Started

### Prerequisites
- Rust (latest stable)
- PostgreSQL
- Docker (optional, for local development)

### Setup
1. Copy `.env.example` to `.env` and edit as needed.
2. Create and migrate the database.
3. Build and run the backend:
   ```sh
   cargo build
   cargo run
   ```

### Configuration
- Environment variables are loaded from `.env` and can be overridden.
- See `.env.example` and `config/default.toml` for options.

### API Endpoints
- `GET /health` - Health check
- `GET /ready` - Readiness check
- `GET /metrics` - Prometheus metrics endpoint
- `GET /api/v1/mutations` - List mutation tests
- `POST /api/v1/mutations` - Create a mutation test
- ... (see code for full list)

### Testing
- Run all tests:
  ```sh
  cargo test
  ```

### Linting & Formatting
- Check formatting:
  ```sh
  cargo fmt --all -- --check
  ```
- Run linter:
  ```sh
  cargo clippy --all-targets --all-features -- -D warnings
  ```

### Observability
- Logs: structured with tracing
- Metrics: Prometheus endpoint
- Tracing: Jaeger/OTLP support

## Mutation Testing & CI Integration

### Running Mutation Tests Locally

To run mutation tests locally:
```sh
cargo run --bin flux-backend --features mutation-testing
```

Mutation reports will be generated in the `backend/mutation-report/` directory.

### GitHub Actions Integration

A sample workflow is provided in `.github/workflows/mutation-testing.yml`:
```yaml
# ...see file for full example...
```
This workflow will build, test, and run mutation testing on every push and pull request to `main`. Mutation reports are uploaded as artifacts.

### GitLab CI Integration

A sample pipeline is provided in `.gitlab-ci.yml`:
```yaml
# ...see file for full example...
```
This pipeline will build, test, and run mutation testing, saving reports as artifacts.

### Custom Configuration

You can configure mutation testing via `flux.config.yaml` or `flux.config.toml` in the project root. Example:
```yaml
timeout_seconds: 60
max_mutations_per_line: 10
test_command: "cargo test -- --test-threads=1"
mutation_types:
  - arithmetic
  - logical
  - boolean
excluded_mutations:
  - string
ast_mutations_enabled: true
```

### Reporting & Visualization

Reports can be generated in JSON, CSV, HTML, or Markdown formats. Visual charts are saved in `mutation-report/`.

### API Usage

See API endpoints above for programmatic access to mutation testing features.

## Contributing
Contributions are welcome! Please open issues or pull requests.

## License
MIT