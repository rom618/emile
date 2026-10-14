use std::sync::Arc;

pub fn release() -> String {
    match std::env::var("EMILE_REVISION").ok().filter(|r| !r.trim().is_empty()) {
        Some(rev) => format!("emile@{}+{}", env!("CARGO_PKG_VERSION"), rev.trim()),
        None => format!("emile@{}", env!("CARGO_PKG_VERSION")),
    }
}

pub fn init() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("EMILE_SENTRY_DSN").ok().map(|d| d.trim().to_string()).filter(|d| !d.is_empty())?;
    let mut options = sentry::ClientOptions::default();
    options.release = Some(release().into());
    options.environment = Some(std::env::var("EMILE_SENTRY_ENVIRONMENT").unwrap_or_else(|_| "production".into()).into());
    options.send_default_pii = false;
    options.max_breadcrumbs = 0;
    options.attach_stacktrace = true;
    options.before_breadcrumb = Some(Arc::new(|_| None));
    let guard = sentry::init((dsn, options));
    guard.is_enabled().then_some(guard)
}

pub fn tracing_layer<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    sentry::integrations::tracing::layer().event_filter(|meta| match *meta.level() {
        tracing::Level::ERROR => sentry::integrations::tracing::EventFilter::Event,
        _ => sentry::integrations::tracing::EventFilter::Ignore,
    })
}

pub fn send_test_error() -> bool {
    sentry::capture_message("emile report-test", sentry::Level::Error);
    sentry::Hub::current().client().map(|c| c.flush(Some(std::time::Duration::from_secs(5)))).unwrap_or(false)
}
