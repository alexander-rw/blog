# ---- Builder stage (used by local builds only) ----
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# 1) Copy only dependency manifests and create stubs so that `cargo build`
#    downloads and compiles all dependencies in a cacheable layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "fn main() {}" > build.rs
RUN cargo build --release
RUN rm -rf src build.rs

# 2) Copy the real source tree (build.rs needs pages/ and assets/ at compile
#    time because it embeds them via include_str!).
COPY build.rs build.rs
COPY src/ src/
COPY pages/ pages/
COPY assets/ assets/

# 3) Touch main.rs so cargo detects the source change and recompiles, reusing
#    the already-built dependencies from the cached layer above.
RUN touch src/main.rs && cargo build --release

# ---- Shared runtime base ----
FROM debian:bookworm-slim AS base

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3084
CMD ["blog"]

# ---- CI target: use pre-built binary from build context ----
FROM base AS ci
COPY blog /usr/local/bin/blog

# ---- Local target (default): copy binary from builder stage ----
FROM base AS local
COPY --from=builder /app/target/release/blog /usr/local/bin/blog
