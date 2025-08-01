# Build stage
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /usr/src/cassh2rs

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tini

# Create non-root user
RUN addgroup -g 1000 cassh2rs && \
    adduser -D -u 1000 -G cassh2rs cassh2rs

# Copy the binary from builder
COPY --from=builder /usr/src/cassh2rs/target/release/cassh2rs /usr/local/bin/cassh2rs

# Set ownership
RUN chown -R cassh2rs:cassh2rs /usr/local/bin/cassh2rs && \
    chmod +x /usr/local/bin/cassh2rs

# Switch to non-root user
USER cassh2rs

# Use tini as entrypoint
ENTRYPOINT ["/sbin/tini", "--"]

# Default command
CMD ["cassh2rs", "--help"]