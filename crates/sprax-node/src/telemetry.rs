use crate::config::TelemetryConfig;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes structured logging and tracing for the node runtime.
pub fn init_telemetry(config: &TelemetryConfig) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    if config.log_format_json {
        let fmt_layer = fmt::layer().json();
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init();
    } else {
        let fmt_layer = fmt::layer().compact().with_target(true);
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init();
    }
}
