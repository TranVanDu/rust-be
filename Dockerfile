# Stage 1: Common build environment
FROM rust:1.83.0-alpine AS common-builder

# Set working directory
WORKDIR /usr/src/app

# Set build arguments (though not used in the build steps, good to keep if needed externally)
ARG TARGETARCH
ARG RUST_VERSION=1.83.0

# Disable Rust auto-updates for reproducible builds
ENV RUSTUP_UPDATE_ROOT=""
ENV RUSTUP_DIST_SERVER=""

# Set environment variables for linking (keep CC, AR, RANLIB)
# REMOVED: ENV RUSTFLAGS="-C target-feature=-crt-static"
ENV CC=musl-gcc
ENV AR=ar
ENV RANLIB=ranlib

# Install build dependencies in one consolidated step
RUN apk add --no-cache \
    musl-dev \
    musl-tools \
    musl-gcc \
    musl-libc-dev \
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
    openssl-dev

# Install cargo-chef and sqlx-cli for dependency caching and migrations
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
COPY migrations ./migrations

# Build the application with optimizations
# REMOVED: -C target-feature=-crt-static from RUSTFLAGS here too
ENV RUSTFLAGS="-C target-cpu=native"
RUN cargo build --release --bin app && \
    # Clean up build artifacts
    rm -rf target/release/build target/release/deps target/release/.fingerprint/app-*

# Stage 4: Runtime - Minimal production image
FROM rust:1.83.0-alpine

# Create non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Install only necessary runtime dependencies
RUN apk add --no-cache \
    libpq \
    libjpeg-turbo \
    openssl \
    libgcc \
    libstdc++ \
    ca-certificates \
    tzdata \
    postgresql-client \
    # Add musl runtime libraries for completeness, though often included by base
    musl

# Install sqlx-cli (needed for migrations in entrypoint)
# This was causing issues in the builder stage, so it's good it's in a later stage now.
# However, if your entrypoint *relies* on sqlx-cli, it must be installed here.
# If the entrypoint is run on the builder image (which is not what you have here),
# then sqlx-cli would only be needed there.
# Since you copy sqlx-cli into the final image via a `cargo install` in Stage 1,
# you don't need to re-install it here in Stage 4.
# Let's remove the re-install here as it's already done in common-builder.
# RUN cargo install sqlx-cli --no-default-features --features postgres # <-- REMOVE THIS LINE

# Create necessary directories and set permissions
RUN mkdir -p /usr/src/app/uploads && \
    mkdir -p /usr/src/app/config && \
    mkdir -p /usr/src/app/migrations && \
    chown -R appuser:appgroup /usr/src/app

# Copy only required files from builder
# The sqlx-cli binary installed in common-builder (Stage 1) will be available in the final image
# because common-builder is the base for planner and builder, and builder is the source for runtime.
# This means /usr/local/cargo/bin is present in the final image.
COPY --from=builder /usr/src/app/target/release/app /usr/local/bin/app
COPY --from=builder /usr/src/app/migrations /usr/src/app/migrations
COPY --from=builder /usr/src/app/backup.sql /usr/src/app/backup.sql
# Ensure these paths exist in the builder stage if they don't, or handle them being optional.
# For example, if uploads is just a runtime directory, you shouldn't copy it from builder.
# If config/firebase-service-account.json is *not* built/generated, it won't exist in builder.
# You might need to add COPY statements for these files from your local context in an earlier stage,
# or handle their absence. Assuming they are in your local context.
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
ENV ENV=production \
    RUST_LOG=info \
    RUST_BACKTRACE=0 \
    TZ=UTC

# Expose port and set healthcheck
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Use entrypoint script
ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["app"]
