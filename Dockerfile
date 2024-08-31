# Stage 1: Build
FROM rust:1.80.1 AS builder

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create a new directory for the project
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Create a dummy file to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Fetch dependencies
RUN cargo build --release

# Copy the actual source code
COPY . .

# Build the actual binary
RUN cargo build --release

# Stage 2: Final image
FROM scratch

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/officer /officer

# Set the entrypoint for the container
ENTRYPOINT ["/officer"]