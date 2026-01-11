//! ShaderJoy Desktop Application Entry Point

use anyhow::Result;
use tracing_subscriber::{
    fmt,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter,
};

fn main() -> Result<()> {
    init_tracing()?;

    tracing::info!("ShaderJoy v{}", env!("CARGO_PKG_VERSION"));

    Ok(())
}

/// Initialize tracing with console-subscriber (for tokio-console) and fmt layer (for stdout).
fn init_tracing() -> Result<()> {
    let console_layer = console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .spawn();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("shaderjoy=debug,info"));

    tracing_subscriber::registry()
        .with(console_layer)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(env_filter),
        )
        .init();

    Ok(())
}
