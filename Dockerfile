### Stage 1: Planner
FROM rust:1.86.0 as planner

WORKDIR /app

# Install cargo-chef
RUN cargo install cargo-chef

# Copy manifests only (no source code)
COPY Cargo.toml Cargo.lock ./

# Create a dummy src to prevent error
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Generate recipe (snapshot dependencies state)
RUN cargo chef prepare --recipe-path recipe.json

### Stage 2: Cacher
FROM rust:1.86.0 as cacher

WORKDIR /app

RUN cargo install cargo-chef

# Copy recipe generated previously
COPY --from=planner /app/recipe.json recipe.json

# Fetch and build dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl

### Stage 3: Builder
FROM rust:1.86.0 as builder

WORKDIR /app

# Install musl-tools for static linking
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add $TARGET

# Copy source code
COPY . .

# Build actual application
RUN cargo build --release --target $TARGET

### Stage 4: Final
FROM scratch

# Copy CA certificates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy built binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/officer /officer

EXPOSE 8000

ENTRYPOINT ["/officer"]
