use std::sync::OnceLock;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_error::ErrorLayer;

static GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let (writer, guard) = non_blocking(std::io::stderr());
    GUARD.set(guard).unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()
            .without_time()
            .with_target(false)
            .with_writer(writer))
        .with(ErrorLayer::default())
        .init();

    tracing_log::LogTracer::init().ok();
}