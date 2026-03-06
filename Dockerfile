FROM rust:1.94-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy sources for all binaries
RUN mkdir -p src/bin/deploy src/bin/gen_node_config src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/deploy/main.rs && \
    echo "fn main() {}" > src/bin/gen_node_config.rs && \
    echo "pub mod adapters; pub mod db; pub mod domain; pub mod service; pub mod utils;" > src/lib.rs

# Build dependencies only (cached)
RUN cargo build --release --locked
RUN rm -rf src

# Copy actual source code
COPY . .

# Build the real application
RUN cargo build --release --locked


# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bot /usr/local/bin/bot

# Optional config files
COPY config/domains.json /app/config/domains.json

# Persistent data directory
RUN mkdir -p /app/data
VOLUME /app/data

EXPOSE 8080

CMD ["bot"]