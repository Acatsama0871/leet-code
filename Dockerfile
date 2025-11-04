# Multi-stage Dockerfile for LeetCode Tracker

# Stage 1: Build Frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy frontend package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Build frontend (outputs to ../backend/static)
RUN npm run build

# Stage 2: Build Backend
FROM rust:1.84-slim AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy backend manifests
COPY backend/Cargo.toml backend/Cargo.lock* ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release || true && \
    rm -rf src

# Copy backend source
COPY backend/src ./src

# Build backend
RUN cargo build --release

# Stage 3: Final Runtime Image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /app/target/release/leetcode-tracker-backend /app/backend

# Copy frontend static files
COPY --from=frontend-builder /app/backend/static /app/static

# Create data directory for DuckDB
RUN mkdir -p /app/data

# Set environment variables
ENV DATABASE_PATH=/app/data/leetcode.duckdb
ENV PORT=3000

# Expose port
EXPOSE 3000

# Run backend (serves both API and static frontend)
CMD ["/app/backend"]
