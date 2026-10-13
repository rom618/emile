pub mod ai;
pub mod items;
pub mod telemetry;

use axum::routing::{get, post};
use axum::Router;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub claude: ai::Claude,
}

pub async fn connect_db(path: &str) -> anyhow::Result<SqlitePool> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{path}"))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(4).connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub fn build_router(state: AppState, static_dir: &str) -> Router {
    let index = format!("{static_dir}/index.html");
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/items", get(items::list).post(items::add))
        .route("/api/items/parse", post(items::parse))
        .fallback_service(ServeDir::new(static_dir).fallback(ServeFile::new(index)))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
