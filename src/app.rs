use std::{borrow::Cow, fs, sync::Arc, time::Instant};

use tokio::{
    runtime::Handle,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    generation::{Generation, Specimen, generate_specimens},
    shader_gen::config::ShaderGenConfig,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    frame: u32,
    mouse_x: f32,
    mouse_y: f32,
    mouse_pressed: u32,
    opacity: f32,
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

enum ShaderSource {
    Code(String),
    PathWithFallback { path: String, fallback: String },
}

struct ComponentSpec {
    shader: ShaderSource,
    label: String,
    rect: Rect,
    opacity: f32,
}

struct ShaderComponent {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    source: String,
    rect: Rect,
    opacity: f32,
}

pub struct App {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    components: Vec<ShaderComponent>,
    bind_group_layout: wgpu::BindGroupLayout,
    background_component: ShaderComponent,
    start_time: Instant,
    fade_start_time: Option<Instant>,
    frame_count: u32,
    mouse_pos: (f32, f32),
    mouse_pressed: bool,
    window: Arc<winit::window::Window>,

    // UI and layout
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    grid_cols: u32,
    grid_rows: u32,

    // Shader generation
    generation: Generation,
    shader_config: ShaderGenConfig,
    generation_rx: UnboundedReceiver<Vec<Specimen>>,
    generation_tx: UnboundedSender<Vec<Specimen>>,
    is_generating: bool,
    runtime: Handle,
}

impl App {
    pub async fn new(window: Arc<winit::window::Window>, runtime: Handle) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let background_component = build_component(
            &device,
            &bind_group_layout,
            config.format,
            ComponentSpec {
                shader: ShaderSource::PathWithFallback {
                    path: "src/starfield.wgsl".to_string(),
                    fallback: include_str!("shader.wgsl").to_string(),
                },
                label: "src/starfield.wgsl".to_string(),
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1.0,
                    h: 1.0,
                },
                opacity: 1.0,
            },
        );

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1);
        let (generation_tx, generation_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut app = Self {
            surface,
            device,
            queue,
            config,
            size,
            components: Vec::new(),
            bind_group_layout,
            background_component,
            start_time: Instant::now(),
            fade_start_time: None,
            frame_count: 0,
            mouse_pos: (0.0, 0.0),
            mouse_pressed: false,
            window,
            egui_ctx,
            egui_state,
            egui_renderer,
            grid_cols: 3,
            grid_rows: 3,
            generation: Generation::new(),
            shader_config: ShaderGenConfig::default(),
            generation_rx,
            generation_tx,
            is_generating: false,
            runtime,
        };

        app.prime_generation().await;
        app.rebuild_components();
        app
    }

    pub fn handle_event(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                let response = self.egui_state.on_window_event(&self.window, &event);
                if response.consumed {
                    return;
                }
                self.handle_window_event(&event, elwt);
            }
            Event::AboutToWait => self.window.request_redraw(),
            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(size) => self.handle_resize(*size),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                if self.mouse_pressed {
                    self.handle_mouse_click();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event),
            WindowEvent::RedrawRequested => self.handle_redraw(),
            _ => {}
        }
    }

    fn handle_redraw(&mut self) {
        self.update();
        if let Err(e) = self.render() {
            eprintln!("{:?}", e);
        }
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.rebuild_components();
    }

    fn handle_key(&mut self, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }

        if key.physical_key == PhysicalKey::Code(KeyCode::KeyR) {
            self.reload_shader();
        }

        if key.physical_key == PhysicalKey::Code(KeyCode::KeyF) {
            self.fade_start_time = Some(Instant::now());
        }
    }

    fn handle_mouse_click(&mut self) {
        if self.is_generating {
            println!("Generation already in progress, ignoring click.");
            return;
        }

        let Some(normalized) = self.normalized_mouse_position() else {
            return;
        };

        let Some(index) = self.hit_test_component(normalized) else {
            return;
        };

        if let Some(parent) = self.generation.current.get(index).cloned() {
            println!("Tile {} clicked. Starting evolution...", index);
            self.start_generation(parent);
        }
    }

    fn start_generation(&mut self, parent: Specimen) {
        let count = (self.grid_cols * self.grid_rows) as usize;
        let config = self.shader_config.clone();
        let tx = self.generation_tx.clone();

        self.is_generating = true;
        self.runtime.spawn(async move {
            let results = generate_specimens(Some(parent), count, &config).await;
            let valid: Vec<Specimen> = results.into_iter().filter_map(|r| r.ok()).collect();

            if let Err(e) = tx.send(valid) {
                eprintln!("Failed to send generation results: {}", e);
            }
        });
    }

    async fn prime_generation(&mut self) {
        self.generation
            .generate(
                None,
                (self.grid_cols * self.grid_rows) as usize,
                &self.shader_config,
            )
            .await;
    }

    fn rebuild_components(&mut self) {
        self.components.clear();

        let fallback_source = include_str!("shader.wgsl").to_string();
        let rects = match self.grid_rects() {
            Some(rects) => rects,
            None => return,
        };

        for (i, rect) in rects.into_iter().enumerate() {
            let (shader, label) = if let Some(specimen) = self.generation.current.get(i) {
                (
                    ShaderSource::Code(specimen.code.clone()),
                    format!("Generated {}", i),
                )
            } else {
                (
                    ShaderSource::Code(fallback_source.clone()),
                    "Fallback".to_string(),
                )
            };

            self.components
                .push(self.build_component_for_state(ComponentSpec {
                    shader,
                    label,
                    rect,
                    opacity: 1.0,
                }));
        }
    }

    fn grid_rects(&self) -> Option<Vec<Rect>> {
        let screen_w = self.config.width as f32;
        let screen_h = self.config.height as f32;

        if screen_w == 0.0 || screen_h == 0.0 {
            return None;
        }

        let max_tile_w = screen_w / self.grid_cols as f32;
        let max_tile_h = screen_h / self.grid_rows as f32;
        let tile_s = max_tile_w.min(max_tile_h);
        let step_x = tile_s / screen_w;
        let step_y = tile_s / screen_h;
        let total_w = tile_s * self.grid_cols as f32;
        let total_h = tile_s * self.grid_rows as f32;
        let offset_x = (screen_w - total_w) / 2.0 / screen_w;
        let offset_y = (screen_h - total_h) / 2.0 / screen_h;

        let mut rects = Vec::with_capacity((self.grid_cols * self.grid_rows) as usize);
        for y in 0..self.grid_rows {
            for x in 0..self.grid_cols {
                rects.push(Rect {
                    x: offset_x + x as f32 * step_x,
                    y: offset_y + y as f32 * step_y,
                    w: step_x,
                    h: step_y,
                });
            }
        }

        Some(rects)
    }

    fn build_component_for_state(&self, spec: ComponentSpec) -> ShaderComponent {
        build_component(
            &self.device,
            &self.bind_group_layout,
            self.config.format,
            spec,
        )
    }

    fn normalized_mouse_position(&self) -> Option<(f32, f32)> {
        if self.config.width == 0 || self.config.height == 0 {
            return None;
        }

        Some((
            self.mouse_pos.0 / self.config.width as f32,
            self.mouse_pos.1 / self.config.height as f32,
        ))
    }

    fn hit_test_component(&self, normalized: (f32, f32)) -> Option<usize> {
        let (mx, my) = normalized;
        self.components.iter().position(|comp| {
            mx >= comp.rect.x
                && mx <= comp.rect.x + comp.rect.w
                && my >= comp.rect.y
                && my <= comp.rect.y + comp.rect.h
        })
    }

    fn reload_shader(&mut self) {
        println!("Reloading shaders...");
        self.rebuild_components();

        self.background_component = self.build_component_for_state(ComponentSpec {
            shader: ShaderSource::PathWithFallback {
                path: "src/starfield.wgsl".to_string(),
                fallback: include_str!("shader.wgsl").to_string(),
            },
            label: "src/starfield.wgsl".to_string(),
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            opacity: 1.0,
        });
        println!("All shaders reloaded.");
    }

    fn calculate_opacity(&mut self) -> f32 {
        const FADE_OUT_SECS: f32 = 2.0;
        const HOLD_BLACK_SECS: f32 = 1.0;

        if let Some(start) = self.fade_start_time {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed < FADE_OUT_SECS {
                1.0 - (elapsed / FADE_OUT_SECS)
            } else if elapsed < FADE_OUT_SECS + HOLD_BLACK_SECS {
                0.0
            } else {
                self.fade_start_time = None;
                1.0
            }
        } else {
            1.0
        }
    }

    fn build_global_uniforms(&self) -> Uniforms {
        Uniforms {
            time: self.start_time.elapsed().as_secs_f32(),
            width: self.size.width as f32,
            height: self.size.height as f32,
            frame: self.frame_count,
            mouse_x: self.mouse_pos.0,
            mouse_y: self.mouse_pos.1,
            mouse_pressed: if self.mouse_pressed { 1 } else { 0 },
            opacity: 1.0,
        }
    }

    fn update(&mut self) {
        self.pump_generation_results();

        let global_uniforms = self.build_global_uniforms();
        let fade_factor = self.calculate_opacity();

        let mut bg_uniforms = global_uniforms;
        bg_uniforms.opacity = fade_factor;
        self.queue.write_buffer(
            &self.background_component.uniform_buffer,
            0,
            bytemuck::cast_slice(&[bg_uniforms]),
        );

        for comp in &self.components {
            let mut u = global_uniforms;
            u.opacity = comp.opacity * fade_factor;
            self.queue
                .write_buffer(&comp.uniform_buffer, 0, bytemuck::cast_slice(&[u]));
        }

        self.frame_count += 1;
    }

    fn pump_generation_results(&mut self) {
        let mut handled = false;

        while let Ok(new_specimens) = self.generation_rx.try_recv() {
            handled = true;
            if new_specimens.is_empty() {
                println!("Generation produced no valid specimens.");
                continue;
            }

            println!("New generation received. Updating grid.");
            self.generation.advance(new_specimens);
            self.rebuild_components();
        }

        if handled {
            self.is_generating = false;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_ctx.begin_frame(raw_input);

        let mut grid_size_changed = false;

        egui::Window::new("Settings").show(&self.egui_ctx, |ui| {
            ui.label("Grid Size");
            if ui
                .add(egui::Slider::new(&mut self.grid_cols, 1..=5).text("Columns"))
                .changed()
            {
                grid_size_changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut self.grid_rows, 1..=5).text("Rows"))
                .changed()
            {
                grid_size_changed = true;
            }
        });

        let full_output = self.egui_ctx.end_frame();

        if grid_size_changed {
            self.rebuild_components();
        }

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            let screen_w = self.config.width as f32;
            let screen_h = self.config.height as f32;

            rpass.set_viewport(0.0, 0.0, screen_w, screen_h, 0.0, 1.0);
            rpass.set_pipeline(&self.background_component.pipeline);
            rpass.set_bind_group(0, &self.background_component.bind_group, &[]);
            rpass.draw(0..3, 0..1);

            for comp in &self.components {
                if let Some((safe_x, safe_y, final_w, final_h)) =
                    clamped_viewport(&comp.rect, screen_w, screen_h)
                {
                    rpass.set_viewport(safe_x, safe_y, final_w, final_h, 0.0, 1.0);
                    rpass.set_pipeline(&comp.pipeline);
                    rpass.set_bind_group(0, &comp.bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                }
            }

            rpass.set_viewport(0.0, 0.0, screen_w, screen_h, 0.0, 1.0);
            self.egui_renderer
                .render(&mut rpass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn build_component(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    spec: ComponentSpec,
) -> ShaderComponent {
    let ComponentSpec {
        shader,
        label,
        rect,
        opacity,
    } = spec;

    let shader_source = match shader {
        ShaderSource::Code(code) => code,
        ShaderSource::PathWithFallback { path, fallback } => fs::read_to_string(&path)
            .unwrap_or_else(|_| {
                println!("Failed to read {}, using fallback.", path);
                fallback
            }),
    };

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&label),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[layout],
        ..Default::default()
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("Uniform Buffer {}", label)),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: None,
    });

    ShaderComponent {
        pipeline,
        uniform_buffer,
        bind_group,
        source: label,
        rect,
        opacity,
    }
}

fn clamped_viewport(rect: &Rect, screen_w: f32, screen_h: f32) -> Option<(f32, f32, f32, f32)> {
    let x = rect.x * screen_w;
    let y = rect.y * screen_h;
    let w = rect.w * screen_w;
    let h = rect.h * screen_h;

    let safe_x = x.max(0.0);
    let safe_y = y.max(0.0);
    let safe_w = w.max(1.0);
    let safe_h = h.max(1.0);

    let final_w = if safe_x + safe_w > screen_w {
        screen_w - safe_x
    } else {
        safe_w
    };

    let final_h = if safe_y + safe_h > screen_h {
        screen_h - safe_y
    } else {
        safe_h
    };

    if final_w > 0.0 && final_h > 0.0 {
        Some((safe_x, safe_y, final_w, final_h))
    } else {
        None
    }
}
