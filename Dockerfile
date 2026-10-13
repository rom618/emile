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
    && rm -rf src target/release/.fingerprint/pantry-cook-* target/release/deps/pantry_cook-* target/release/deps/libpantry_cook-*
COPY backend/src ./src
COPY backend/migrations ./migrations
RUN cargo build --release --locked

FROM node:24.21.0-bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g --no-audit --no-fund @anthropic-ai/claude-code@2.1.285 \
    && useradd --create-home --uid 10001 pantry \
    && mkdir -p /app/app-data && chown pantry:pantry /app/app-data
WORKDIR /app
COPY --from=backend-build /app/backend/target/release/pantry-cook ./pantry-cook
COPY --from=frontend-build /app/frontend/dist ./static

ENV PANTRY_STATIC_DIR=/app/static
ENV PANTRY_DB_PATH=/app/app-data/pantry.db
ENV PANTRY_BIND=0.0.0.0:8080
ENV DISABLE_AUTOUPDATER=1
ENV CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
ARG PANTRY_REVISION=
ENV PANTRY_REVISION=$PANTRY_REVISION

USER pantry
EXPOSE 8080
CMD ["./pantry-cook"]
