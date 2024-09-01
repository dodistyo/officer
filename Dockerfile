# Stage 1: Build
FROM rust:1.80.1 AS builder

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    musl-tools \
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
RUN ls -lah
# Stage 2: Final image
FROM scratch

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/officer /officer
# Expose port
EXPOSE 8000
# Set the entrypoint for the container
ENTRYPOINT ["/officer"]