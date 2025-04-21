# Stage 1: Build
FROM rust:1.86.0 AS builder

# Install dependencies including CA certificates
RUN apt-get update && apt-get install -y \
    musl-tools ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the environment for musl
ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add $TARGET

# Create a new directory for the project
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Create a dummy file to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Fetch dependencies
RUN cargo build --release --target $TARGET

# Copy the actual source code
COPY . .

# Build the actual binary
RUN cargo build --release --target $TARGET

# Stage 2: Final image
FROM scratch
# Copy CA certificates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/officer /officer

# Expose port
EXPOSE 8000

# Set the entrypoint for the container
ENTRYPOINT ["/officer"]
