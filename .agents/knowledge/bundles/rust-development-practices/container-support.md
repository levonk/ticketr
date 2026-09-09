---
type: Practice
title: Container Support
description: Multi-stage Dockerfile for minimal runtime images, non-root user creation, healthcheck, and docker-compose integration for Rust packages.
tags: [rust, docker, container, multi-stage, healthcheck, docker-compose]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
---

# Container Support

## Failure Mode

Single-stage Dockerfiles ship the entire Rust toolchain in the runtime image.
Root containers allow privilege escalation. Missing healthchecks prevent
orchestration systems from detecting dead containers.

## Practice

### Multi-Stage Dockerfile

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false rustuser
WORKDIR /app
COPY --from=builder /app/target/release/package /usr/local/bin/
USER rustuser
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD package --health-check || exit 1
ENTRYPOINT ["package"]
```

### Docker Compose

```yaml
services:
  package:
    build: .
    restart: unless-stopped
    environment:
      - RUST_LOG=info
    ports:
      - "8080:8080"
    healthcheck:
      test: ["CMD", "package", "--health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## Related Concepts

- [CLI Tool Standards](cli-tool-standards.md) — Health check mechanism for containers
- [Security and Auditing](security-auditing.md) — Non-root user, minimal image
