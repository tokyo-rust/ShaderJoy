//! ShaderJoy Desktop Application Entry Point

use anyhow::Result;
use shaderjoy_core::{config::AppConfig, describe_config_paths};
use tracing::info;
use tracing_subscriber::{
    fmt,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter,
};

fn main() -> Result<()> {
    init_tracing()?;

    let (config, is_default) = AppConfig::load()?;
    if is_default {
        info!("Using default config");
        info!("{}", describe_config_paths());
    }

    info!("Using Config: {:#?}", config);

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

    info!("ShaderJoy v{}", env!("CARGO_PKG_VERSION"));

    Ok(())
}
