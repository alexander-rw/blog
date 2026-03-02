# ---- Builder stage (used by local builds only) ----
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# 1) Copy manifests + real build.rs. Stub only the Rust source with an empty
#    pages/ dir. The real build.rs handles empty pages/ gracefully (collect_mdx
#    returns early), so it writes a valid-but-empty generated_pages.rs to
#    OUT_DIR. This lets all dependencies compile and be cached in this layer.
COPY Cargo.toml Cargo.lock build.rs ./
RUN mkdir -p src pages && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 2) Copy the real source tree and content files.
COPY src/ src/
COPY pages/ pages/
COPY assets/ assets/

# 3) Recompile: src/main.rs changed (stub -> real), and cargo detects pages/
#    changed via rerun-if-changed, so the build script re-runs and regenerates
#    generated_pages.rs with real content before the binary is compiled.
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
