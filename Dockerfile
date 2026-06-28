# syntax=docker/dockerfile:1

# =============================================================================
# Stage 1a -- gui-builder  (skipped when building --target runtime)
# Compiles the React/Vite frontend. Node.js is not needed in the final image.
# =============================================================================
FROM node:22-slim AS gui-builder

WORKDIR /gui
COPY gui/package.json ./
# TODO: commit a package-lock.json and switch back to `npm ci` for
# reproducible CI/release builds. Currently uses npm install because
# gui/ has no standard lockfile (only frameshift.lock).
RUN npm install
COPY gui/ ./
RUN npm run build

# =============================================================================
# Stage 1b -- builder
# Compiles kleos-server and kleos-cli in release mode.
# SQLCipher is vendored at compile time via the "sqlcipher" feature so no
# system libsqlcipher is needed at runtime.
# =============================================================================
FROM rust:1.94-bookworm AS builder

WORKDIR /build

# Install build-time deps needed by vendored SQLCipher and OpenSSL bindings.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    clang \
    protobuf-compiler \
    libprotobuf-dev \
    libpcsclite-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the full workspace so Cargo can resolve the dependency graph.
COPY . .

# Cap release-build memory so it fits the 16 GB CI runners. Fat LTO with a
# single codegen unit was SIGKILLed (OOM) while compiling kleos-server on the
# arm64 runner; thin LTO with 16 codegen units builds within memory and faster,
# with negligible runtime cost for a server binary. Scoped to this build only.
ENV CARGO_PROFILE_RELEASE_LTO=thin \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16

# Build with BuildKit cache mounts so the Cargo registry and compiled
# dependencies survive across rebuilds — only changed crates are recompiled.
# Binaries are copied to /tmp here because cache mounts are not accessible
# from other stages.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release -p kleos-server -p kleos-cli -p kleos-sidecar -p kleos-credd -p kleos-phylaxd \
    && cp target/release/kleos-server  /tmp/kleos-server \
    && cp target/release/kleos-cli     /tmp/kleos-cli \
    && cp target/release/kleos-sidecar /tmp/kleos-sidecar \
    && cp target/release/kleos-credd   /tmp/kleos-credd \
    && cp target/release/phylaxd       /tmp/phylaxd

# =============================================================================
# Stage 2 -- runtime
# Minimal Debian image with only the libraries the binaries actually dlopen.
# =============================================================================
FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.source="https://github.com/Ghost-Frame/Kleos" \
      org.opencontainers.image.description="Kleos memory server (formerly Engram) -- personal knowledge graph and semantic memory store" \
      org.opencontainers.image.licenses="Elastic-2.0"

# Install runtime dependencies:
#   libssl3     -- required by reqwest (native-tls)
#   ca-certificates -- required for outbound HTTPS calls
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    libpcsclite1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a dedicated non-root user for running the server.
RUN groupadd --system --gid 1000 kleos \
    && useradd --system --uid 1000 --gid kleos --no-create-home --shell /sbin/nologin kleos

# Persistent data lives here.  A named volume or bind-mount should be attached.
RUN mkdir -p /data && chown kleos:kleos /data

COPY --from=builder /tmp/kleos-server /usr/local/bin/kleos-server
COPY --from=builder /tmp/kleos-cli   /usr/local/bin/kleos-cli

RUN chmod 755 /usr/local/bin/kleos-server /usr/local/bin/kleos-cli

# GUI bundle — only present in the `runtime-gui` target.
# COPY --from=gui-builder /gui/build /app/gui/build

# Legacy aliases for backward compatibility.
RUN ln -s /usr/local/bin/kleos-server /usr/local/bin/engram-server \
    && ln -s /usr/local/bin/kleos-cli /usr/local/bin/engram-cli

USER kleos

# Environment -- bind to all interfaces inside the container.
# KLEOS_* vars are preferred. The env shim falls back to ENGRAM_* automatically.
ENV KLEOS_HOST=0.0.0.0
ENV KLEOS_DATA_DIR=/data
ENV KLEOS_DB_PATH=/data/kleos.db

VOLUME ["/data"]

EXPOSE 4200

CMD ["/usr/local/bin/kleos-server"]

# =============================================================================
# Stage 2b -- runtime-gui
# Extends runtime with the pre-built React frontend.
# Build: docker build --target runtime-gui -t kleos:gui .
# =============================================================================
FROM runtime AS runtime-gui

COPY --from=gui-builder /gui/build /app/gui/build
ENV KLEOS_GUI_BUILD_DIR=/app/gui/build

# =============================================================================
# Stage 3 -- sidecar runtime
# Separate, smaller image containing only kleos-sidecar. It holds no DB --
# it's a stateless batching/recall proxy in front of the main server, so it
# gets its own minimal runtime instead of riding along with kleos-server.
# =============================================================================
FROM debian:bookworm-slim AS sidecar

LABEL org.opencontainers.image.source="https://github.com/Ghost-Frame/Kleos" \
      org.opencontainers.image.description="Kleos sidecar -- local batching proxy for observations, session recall, and Claude session-file watching" \
      org.opencontainers.image.licenses="Elastic-2.0"

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    libpcsclite1 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 1000 kleos \
    && useradd --system --uid 1000 --gid kleos --no-create-home --shell /sbin/nologin kleos

COPY --from=builder /tmp/kleos-sidecar /usr/local/bin/kleos-sidecar
RUN chmod 755 /usr/local/bin/kleos-sidecar

USER kleos

# Bind-all is required for Docker's port mapping to reach the process; the
# sidecar's own auth middleware enforces KLEOS_SIDECAR_TOKEN whenever the
# bind host isn't loopback, so this is safe as long as the token is set.
ENV KLEOS_SIDECAR_HOST=0.0.0.0

EXPOSE 7711

CMD ["/usr/local/bin/kleos-sidecar"]

# =============================================================================
# Stage 4 -- credd runtime
# Credential daemon — manages secrets and API key resolution.
# =============================================================================
FROM debian:bookworm-slim AS credd

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    libpcsclite1 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 1000 kleos \
    && useradd --system --uid 1000 --gid kleos --no-create-home --shell /sbin/nologin kleos

COPY --from=builder /tmp/kleos-credd /usr/local/bin/kleos-credd
RUN chmod 755 /usr/local/bin/kleos-credd

USER kleos

ENV CREDD_LISTEN=0.0.0.0:4400

EXPOSE 4400

CMD ["/usr/local/bin/kleos-credd"]

# =============================================================================
# Stage 5 -- phylaxd runtime
# Phylax security daemon — approval gating and policy enforcement.
# =============================================================================
FROM debian:bookworm-slim AS phylaxd

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    libpcsclite1 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 1000 kleos \
    && useradd --system --uid 1000 --gid kleos --no-create-home --shell /sbin/nologin kleos

COPY --from=builder /tmp/phylaxd /usr/local/bin/phylaxd
RUN chmod 755 /usr/local/bin/phylaxd

USER kleos

CMD ["/usr/local/bin/phylaxd"]
