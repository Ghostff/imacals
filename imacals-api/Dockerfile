# syntax=docker/dockerfile:1.7

# ---- Dev stage (with hot reload) ----
FROM rust:1.95.0 AS dev
WORKDIR /app

ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Install build deps and cargo-watch for hot reload
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create user/group dynamically
RUN groupadd --gid $USER_GID developer && \
    useradd -u $USER_UID -g developer -m developer && \
    chown -R developer:developer .

# to use `cargo-watch` in our CMD we need to install cargo-watch CLI into PATH
RUN cargo install --locked cargo-watch && \
    cargo install --locked sqlx-cli --no-default-features --features rustls,postgres

# Copy project files for dev stage
COPY --chown=developer:developer . .

RUN chmod +x ./scripts/migration.sh

# Dev/hot reload runtime settings
ENV FORCE_COLOR=1
ENV TERM=xterm-256color
ENV COLORTERM=truecolor
ENV RUST_BACKTRACE=1
EXPOSE 3032
# Run migrations on boot in dev, then start hot reload
CMD ["/bin/sh", "-c", "sqlx migrate run --source ./src/migrations && cargo watch -x run"]



# ---- Build stage ----
FROM rust:1.95.0 AS builder
WORKDIR /app

# Install build deps
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install sqlx-cli so we can copy it into the runtime image
RUN cargo install --locked sqlx-cli --no-default-features --features rustls,postgres

# Cache deps
COPY Cargo.toml ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN cargo build --release || true

# Now copy full sources
COPY . .
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
RUN useradd -m -u 10001 appuser && \
    apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary (ensure the name matches your Cargo package/bin)
COPY --from=builder /app/target/release/imacals-api /usr/local/bin/imacals-api
# Copy sqlx CLI from builder for running migrations in production
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
# Copy migrations into the runtime image
COPY --from=builder /app/src/migrations /app/src/migrations

# Ensure executables have proper permissions
RUN chmod +x /usr/local/bin/imacals-api /usr/local/bin/sqlx

EXPOSE 3032
USER appuser
ENV RUST_BACKTRACE=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3032 || exit 1

# Run migrations on boot in production, then start the API
CMD ["/bin/sh", "-c", "sqlx migrate run --source ./src/migrations && /usr/local/bin/imacals-api"]
