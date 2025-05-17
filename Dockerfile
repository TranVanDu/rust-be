# Stage 1: Common build environment
FROM rust:1.83.0-alpine AS common-builder
WORKDIR /usr/src/app
ARG TARGETARCH
RUN echo "Building common-builder for TARGETARCH: ${TARGETARCH}"
RUN apk add --no-cache \
    musl-dev \
    postgresql-dev \
    pkgconf \
    libjpeg-turbo-dev \
    openssl-dev \
    g++ \
    cmake \
    make \
    nasm \
    libtool \
    curl
RUN cargo install cargo-chef --locked

# Stage 2: Planner - Calculate dependency blueprint for the workspace
FROM common-builder AS planner
WORKDIR /usr/src/app
ARG TARGETARCH
RUN echo "Building planner for TARGETARCH: ${TARGETARCH}"

COPY Cargo.toml Cargo.lock ./

COPY crates ./crates

COPY migrations ./migrations  

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Compile dependencies and application
FROM common-builder AS builder
WORKDIR /usr/src/app
ARG TARGETARCH
RUN echo "Building builder for TARGETARCH: ${TARGETARCH}"

COPY --from=planner /usr/src/app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --bin app

# Stage 4: Runtime - Minimal production image
FROM alpine:3.20
ARG TARGETARCH
RUN echo "Creating runtime image for TARGETARCH: ${TARGETARCH}"

RUN addgroup -S appgroup && adduser -S appuser -G appgroup

RUN apk add --no-cache \
    libpq \
    libjpeg-turbo \
    openssl \
    libgcc \
    libstdc++

COPY --from=builder /usr/src/app/target/release/app /usr/local/bin/app

RUN chmod +x /usr/local/bin/app

USER appuser

ENV ENV=production
ENV RUST_LOG=info
EXPOSE 8080
CMD ["app"]
