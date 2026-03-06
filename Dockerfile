FROM rust:1.94-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# dependency cache
COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

# real sources
COPY . .

RUN cargo build --release --locked


FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bot /usr/local/bin/bot

COPY config/domains.json /app/config/domains.json

# directory for persistent data
RUN mkdir -p /app/data

VOLUME /app/data

EXPOSE 8080

CMD ["bot"]