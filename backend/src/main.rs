use pantry_cook::{ai::Claude, build_router, connect_db, telemetry, AppState};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

fn env(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.into())
}

fn main() -> anyhow::Result<()> {
    let reporting = telemetry::init();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))))
        .with(reporting.as_ref().map(|_| telemetry::tracing_layer()))
        .init();
    if std::env::args().nth(1).as_deref() == Some("report-test") {
        anyhow::ensure!(reporting.is_some(), "PANTRY_SENTRY_DSN is not set");
        anyhow::ensure!(telemetry::send_test_error(), "the test error could not be delivered");
        println!("test error sent ({})", telemetry::release());
        return Ok(());
    }
    tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(async {
        let db = connect_db(&env("PANTRY_DB_PATH", "./app-data/pantry.db")).await?;
        let state = AppState { db, claude: Claude::from_env() };
        let app = build_router(state, &env("PANTRY_STATIC_DIR", "../frontend/dist"));
        let bind = env("PANTRY_BIND", "127.0.0.1:8080");
        let listener = tokio::net::TcpListener::bind(&bind).await?;
        tracing::info!("listening on {bind} ({})", telemetry::release());
        axum::serve(listener, app).await?;
        Ok(())
    })
}
