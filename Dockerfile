# Stage 1: Common build environment
FROM rust:1.83.0-alpine AS common-builder

# Set working directory
WORKDIR /usr/src/app

# Set build arguments
ARG TARGETARCH
ARG RUST_VERSION=1.83.0

# Disable Rust auto-updates for reproducible builds
ENV RUSTUP_UPDATE_ROOT=""
ENV RUSTUP_DIST_SERVER=""

# Set environment variables for linking
ENV CC=gcc
ENV AR=ar
ENV RANLIB=ranlib
ENV RUSTFLAGS="-C target-cpu=native"
ENV OPENSSL_DIR=/usr
ENV OPENSSL_INCLUDE_DIR=/usr/include
ENV OPENSSL_LIB_DIR=/usr/lib
ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    gcc \
    g++ \
    cmake \
    make \
    nasm \
    libtool \
    curl \
    linux-headers \
    build-base \
    pkgconf \
    postgresql-dev \
    libjpeg-turbo-dev \
    openssl-dev \
    git \
    perl

# Install cargo-chef and sqlx-cli
RUN cargo install cargo-chef --locked && \
    cargo install sqlx-cli --no-default-features --features postgres

# Stage 2: Planner - Calculate dependency blueprint
FROM common-builder AS planner
WORKDIR /usr/src/app

# Copy only the files needed for dependency resolution
COPY Cargo.toml ./
COPY crates ./crates
COPY migrations ./migrations

# Generate dependency recipe
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Compile dependencies and application
FROM common-builder AS builder
WORKDIR /usr/src/app

# Copy dependency recipe from planner
COPY --from=planner /usr/src/app/recipe.json recipe.json

# Build dependencies first (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and migrations
COPY . .

# Build the application
RUN cargo build --release --bin app && \
    # Clean up build artifacts
    rm -rf target/release/build target/release/deps target/release/.fingerprint/app-*

# Stage 4: Runtime - Minimal production image
FROM rust:1.83.0-alpine

# Create non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Install runtime dependencies
RUN apk add --no-cache \
    libpq \
    libjpeg-turbo \
    openssl \
    libgcc \
    libstdc++ \
    ca-certificates \
    tzdata \
    postgresql-client \
    musl-dev \
    gcc \
    openssl-dev \
    postgresql-dev \
    pkgconfig \
    make \
    cmake \
    git \
    curl \
    build-base

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features postgres

# Create necessary directories
RUN mkdir -p /usr/src/app/uploads && \
    mkdir -p /usr/src/app/config && \
    mkdir -p /usr/src/app/migrations && \
    chown -R appuser:appgroup /usr/src/app

# Set working directory
WORKDIR /usr/src/app

# Copy only required files from builder
COPY --from=builder /usr/src/app/target/release/app /usr/local/bin/app
COPY --from=builder /usr/src/app/migrations /usr/src/app/migrations
COPY --from=builder /usr/src/app/backup.sql /usr/src/app/backup.sql
COPY uploads /usr/src/app/uploads 
COPY config/firebase-service-account.json /usr/src/app/config/firebase-service-account.json 

# Set permissions
RUN chmod +x /usr/local/bin/app && \
    chown -R appuser:appgroup /usr/src/app/uploads && \
    chown -R appuser:appgroup /usr/src/app/migrations && \
    chown appuser:appgroup /usr/src/app/backup.sql && \
    chown appuser:appgroup /usr/src/app/config/firebase-service-account.json && \
    chmod 600 /usr/src/app/config/firebase-service-account.json

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Switch to non-root user
USER appuser

# Set environment variables
ENV ENV=production
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0
ENV TZ=UTC

# Expose port and set healthcheck
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080 || exit 1

# Use entrypoint script
ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["/usr/local/bin/app"]
