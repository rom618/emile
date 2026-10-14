FROM node:24.21.0-bookworm-slim AS frontend-build
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY frontend/ ./
RUN npm run build

FROM rust:1.98.1-slim-bookworm AS backend-build
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src target/release/.fingerprint/emile-* target/release/deps/emile-* target/release/deps/libemile-*
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release --locked

FROM node:24.21.0-bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g --no-audit --no-fund @anthropic-ai/claude-code@2.1.285 \
    && useradd --create-home --uid 10001 emile \
    && mkdir -p /app/app-data && chown emile:emile /app/app-data
WORKDIR /app
COPY --from=backend-build /app/backend/target/release/emile ./emile
COPY --from=frontend-build /app/frontend/dist ./static

ENV EMILE_STATIC_DIR=/app/static
ENV EMILE_DB_PATH=/app/app-data/emile.db
ENV EMILE_BIND=0.0.0.0:8080
ENV DISABLE_AUTOUPDATER=1
ENV CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ARG EMILE_REVISION=
ENV EMILE_REVISION=$EMILE_REVISION

USER emile
EXPOSE 8080
CMD ["./emile"]
