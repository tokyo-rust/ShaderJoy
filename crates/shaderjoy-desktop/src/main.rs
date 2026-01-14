//! ShaderJoy Desktop Application Entry Point

use anyhow::Result;
use iced::{window, Settings, Size};
use shaderjoy_core::{config::AppConfig, describe_config_paths};
use tracing::{info, warn};
use tracing_subscriber::{
    fmt,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
    EnvFilter,
};

mod app;
mod shader_widget;
mod ui;

use app::ShaderJoyApp;

fn main() -> Result<()> {
    init_tracing()?;

    let args = std::env::args().collect::<Vec<String>>();
    let (cfg, is_default) = load_cfg(&args)?;

    if is_default {
        info!("Using default config");
        info!("{}", describe_config_paths());
    }

    info!("Using Config: {:#?}", cfg);

    let grid_size = cfg.grid_size;
    let window_width = (grid_size.cols as f32 * 200.0 + 50.0).max(600.0);
    let window_height = (grid_size.rows as f32 * 200.0 + 150.0).max(500.0);

    let settings = Settings {
        antialiasing: true,
        ..Settings::default()
    };

    let window_settings = window::Settings {
        size: Size::new(window_width, window_height),
        min_size: Some(Size::new(400.0, 300.0)),
        ..Default::default()
    };

    iced::application(
        move || ShaderJoyApp::new(cfg.clone()),
        ShaderJoyApp::update,
        ShaderJoyApp::view,
    )
    .subscription(ShaderJoyApp::subscription)
    .theme(ShaderJoyApp::theme)
    .title(ShaderJoyApp::title)
    .settings(settings)
    .window(window_settings)
    .run()?;

    Ok(())
}

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

fn load_cfg(args: &[String]) -> anyhow::Result<(AppConfig, bool)> {
    let (mut cfg, is_default) = AppConfig::load()?;
    if args.iter().any(|a| a == "--test") {
        warn!("Test mode active, using fake LLM provider");
        cfg.test = true
    }
    Ok((cfg, is_default))
}
