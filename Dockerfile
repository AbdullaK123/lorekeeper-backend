FROM node:24-bookworm-slim AS playground
WORKDIR /playground
COPY playground/package.json playground/package-lock.json ./
RUN npm ci
COPY playground/ ./
RUN npm run build

FROM rust:1.98.1-bookworm AS api-build
# Keep this path equal to the runtime path: static serving uses CARGO_MANIFEST_DIR.
WORKDIR /app
ENV SQLX_OFFLINE=true CARGO_INCREMENTAL=0
COPY Cargo.toml Cargo.lock ./
COPY .sqlx/ ./.sqlx/
COPY src/ ./src/
COPY templates/ ./templates/
RUN cargo build --locked --release --bin lorekeeper-backend --jobs 2 \
    && cp target/release/lorekeeper-backend /usr/local/bin/lorekeeper-backend

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /usr/sbin/nologin lorekeeper
WORKDIR /app
COPY --from=api-build /usr/local/bin/lorekeeper-backend /usr/local/bin/lorekeeper-backend
COPY src/app/config/config.yaml ./src/app/config/config.yaml
COPY prompts/ ./prompts/
COPY --from=playground /playground/dist/ ./playground/dist/
ENV ENV=prod
USER lorekeeper
EXPOSE 8000
# The current server handles Ctrl+C; use the same signal for container shutdown.
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/lorekeeper-backend"]
