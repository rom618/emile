# pantry-cook

rom's kitchen app (working name): what is in the pantry, what to cook tonight. Decisions and the
plan: 9s `topics/pantry-cook.md`.

Status: first slice. Pantry list and adding food from a typed note (Claude Haiku reads it, you
check the items before saving). Not deployed.

## Layout

- `backend/`: Rust (axum, sqlx, SQLite). Migrations in `backend/migrations`, applied at start.
- `frontend/`: React + Vite + Tailwind, a page meant to be added to the iPhone home screen.
- `Dockerfile`: builds both and installs Claude Code in the image for the AI calls.

## Run locally

```bash
cd backend && cargo run
cd frontend && npm install && npm run dev
```

The backend listens on `127.0.0.1:8080`, the Vite dev server proxies `/api` to it. Data goes to
`backend/app-data/pantry.db`.

Tests: `cd backend && cargo test` (no AI call is made by the tests).

## Image

```bash
docker build --network host -t pantry-cook:dev .
```

On mikaela the build hung at `npm ci` for over 5 minutes without `--network host` and passed in
seconds with it (2026-10-13, cause not diagnosed; aether's `bin/deploy` uses the same flag). Not deployed to roberta yet.

## Environment

| Variable | Default | Use |
|---|---|---|
| `PANTRY_DB_PATH` | `./app-data/pantry.db` | SQLite file |
| `PANTRY_STATIC_DIR` | `../frontend/dist` | built frontend |
| `PANTRY_BIND` | `127.0.0.1:8080` | listen address |
| `PANTRY_SENTRY_DSN` | unset | Ciel app DSN; errors are reported only when set |
| `PANTRY_SENTRY_ENVIRONMENT` | `production` | |
| `PANTRY_REVISION` | unset | git revision in the release name |
| `PANTRY_CLAUDE_BIN` | `claude` | Claude Code binary |
| `PANTRY_CLAUDE_TIMEOUT_SECS` | `120` | per AI call |
| `ANTHROPIC_BASE_URL` | unset | aether, e.g. `http://aether:8080/v1/claude-code` on roberta |
| `ANTHROPIC_API_KEY` | unset | the app's aether client key `app-pantry-cook` (never printed) |

`backend/pantry-cook report-test` sends one test error to the DSN.

## AI calls

Every call is one `claude -p --model haiku|sonnet --output-format json --tools "" --no-session-persistence --append-system-prompt ...`
run (`backend/src/ai.rs`). The model is a Rust enum with only Haiku and Sonnet, so nothing larger
can be asked for. The system prompt asks for JSON only; the `result` text is parsed (code fences
stripped).

Gotchas:

- Never call aether's subscription route with a plain Messages API request: Anthropic refuses it
  and aether benches the whole account for hours. Only genuine Claude Code (`claude -p`) goes
  there.
- Run locally with `ANTHROPIC_BASE_URL=https://aether.wuling.dev/v1/claude-code` and a key in
  `ANTHROPIC_API_KEY`; without them `claude` uses your normal login.
- Not verified yet (2026-10-13): `--json-schema`, `--setting-sources ""` and
  `--strict-mcp-config`. A test call with all three hung until the timeout, while the flags above
  answered in about 4 s. Bisect them before using any of them.
