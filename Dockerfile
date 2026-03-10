# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Install dependencies for SurrealDB
RUN apt-get update && apt-get install -y \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY static ./static

# Touch main.rs to trigger rebuild
RUN touch src/main.rs

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 portfolio

# Copy the binary
COPY --from=builder /app/target/release/portfolio /app/portfolio

# Copy templates and static files
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static

# Create data directory for SurrealDB
RUN mkdir -p /app/data && chown -R portfolio:portfolio /app

USER portfolio

# Expose port
EXPOSE 8080

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info
ENV DATABASE_URL=file://data/portfolio.db

# Run the application
CMD ["./portfolio"]
