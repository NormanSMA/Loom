FROM node:24-slim AS web-builder
WORKDIR /app
RUN corepack enable && corepack prepare pnpm@11.1.1 --activate
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/web/package.json ./apps/web/
RUN pnpm install --frozen-lockfile
COPY apps/web ./apps/web
RUN pnpm web:build

FROM rust:1-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p loomd -p loom-cli

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/loomd /usr/local/bin/loomd
COPY --from=rust-builder /app/target/release/loom /usr/local/bin/loom
COPY --from=web-builder /app/apps/web/dist /app/web
EXPOSE 4320 4321
CMD ["loomd", "--static-dir", "/app/web"]
