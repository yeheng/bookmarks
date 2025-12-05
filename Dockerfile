# Multi-stage build for Rust backend
FROM rust:1.75 as backend-builder
WORKDIR /app/backend

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy backend source
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual backend source
COPY backend/src ./src
COPY backend/migrations ./migrations
COPY backend/config ./config

# Build backend
RUN touch src/main.rs && cargo build --release

# Multi-stage build for frontend
FROM node:20 as frontend-builder
WORKDIR /app/frontend

# Copy frontend package files
COPY frontend/package*.json ./
RUN npm ci

# Copy frontend source and build
COPY frontend/ ./
RUN npm run build

# Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false app

WORKDIR /app

# Copy backend binary from builder stage
COPY --from=backend-builder /app/backend/target/release/bookmarks-api /app/

# Copy frontend dist from builder stage
COPY --from=frontend-builder /app/frontend/dist /app/frontend/

# Copy migrations and config
COPY backend/migrations /app/migrations
COPY backend/config /app/config

# Create necessary directories
RUN mkdir -p /app/data /app/logs && \
    chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///app/data/bookmarks.db
ENV SERVER_PORT=3000

# Run the application
CMD ["./bookmarks-api"]