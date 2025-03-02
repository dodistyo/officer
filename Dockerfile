# Stage 1: Build
FROM rust:1.85.0 AS builder

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
FROM alpine:3.21
# Install kubectl (get latest version: $(curl -L -s https://dl.k8s.io/release/stable.txt))
ENV KUBECTL_VERSION="v1.32.2"
RUN apk add --no-cache \
    curl \
    bash \
    ca-certificates \
    && curl -LO "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl" \
    && chmod +x kubectl \
    && mv kubectl /usr/local/bin/

# Verify installation
RUN kubectl version --client

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/officer /officer
# Expose port
EXPOSE 8000
# Set the entrypoint for the container
ENTRYPOINT ["/officer"]