// #######################################
// Crate Imports
// #######################################

use std::sync::Arc;

use winit::{event_loop::EventLoop, window::WindowBuilder};

// #######################################
// Module Declarations
// #######################################

mod api;
mod app;
mod generation;
mod shader_gen;

// #######################################
// Application Entrypoint
// #######################################

fn main() {

    // ##############################
    // Window and Event Loop Setup
    // ##############################
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Pick a Shader That Sparks Joy")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .unwrap(),
    );

    // ##############################
    // Runtime Construction
    // ##############################
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // ##############################
    // Server Mode Shortcut
    // ##############################
    if std::env::args().any(|arg| arg == "--serve") {
        rt.block_on(api::run_server("0.0.0.0:7562".parse().unwrap()));
        return;
    }

    // ##############################
    // App Initialization
    // ##############################
    let rt_handle = rt.handle().clone();
    let mut app = rt.block_on(app::App::new(window.clone(), rt_handle));

    // ##############################
    // Event Loop Execution
    // ##############################
    event_loop
        .run(move |event, elwt| {
            app.handle_event(event, elwt);
        })
        .unwrap();
}
