use std::borrow::Cow;
use std::fs;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
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

struct ShaderComponent {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    source: String,
    rect: Rect,
    opacity: f32,
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    components: Vec<ShaderComponent>,
    bind_group_layout: wgpu::BindGroupLayout,
    background_component: ShaderComponent,
    start_time: Instant,
    fade_start_time: Option<Instant>,
    frame_count: u32,
    mouse_pos: (f32, f32),
    mouse_pressed: bool,
    window: Arc<winit::window::Window>,

    // Egui fields
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    grid_cols: u32,
    grid_rows: u32,
}

impl State {
    async fn new(window: Arc<winit::window::Window>) -> State {
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

        // Initialize helper to create components
        let create_comp_helper = |dev: &wgpu::Device,
                                  conf: &wgpu::SurfaceConfiguration,
                                  layout: &wgpu::BindGroupLayout,
                                  src: String,
                                  r: Rect,
                                  op: f32|
         -> ShaderComponent {
            let shader_source = fs::read_to_string(&src).unwrap_or_else(|_| {
                println!("Failed to read {}, using fallback.", src);
                include_str!("shader.wgsl").to_string()
            });

            let shader = dev.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&src),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
            });

            let pipeline_layout = dev.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[layout],
                ..Default::default()
            });

            let pipeline = dev.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&src),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: conf.format,
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

            let uniform_buffer = dev.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Uniform Buffer {}", src)),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = dev.create_bind_group(&wgpu::BindGroupDescriptor {
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
                source: src,
                rect: r,
                opacity: op,
            }
        };

        // Create Background Component
        let background_component = create_comp_helper(
            &device,
            &config,
            &bind_group_layout,
            "src/starfield.wgsl".to_string(),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            1.0,
        );

        // Egui initialization
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1);

        let mut state = Self {
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
        };

        state.rebuild_components();
        state
    }

    fn rebuild_components(&mut self) {
        self.components.clear();
        let cols = self.grid_cols as usize;
        let rows = self.grid_rows as usize;

        let screen_w = self.config.width as f32;
        let screen_h = self.config.height as f32;

        if screen_w == 0.0 || screen_h == 0.0 {
            return;
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

        let source_path = "src/shader.wgsl";

        for y in 0..rows {
            for x in 0..cols {
                let rect = Rect {
                    x: offset_x + x as f32 * step_x,
                    y: offset_y + y as f32 * step_y,
                    w: step_x,
                    h: step_y,
                };

                let component = self.create_component(
                    &self.bind_group_layout,
                    source_path.to_string(),
                    rect,
                    1.0,
                );
                self.components.push(component);
            }
        }
    }

    fn create_component(
        &self,
        layout: &wgpu::BindGroupLayout,
        source: String,
        rect: Rect,
        opacity: f32,
    ) -> ShaderComponent {
        let shader_source = fs::read_to_string(&source).unwrap_or_else(|_| {
            println!("Failed to read {}, using fallback.", source);
            include_str!("shader.wgsl").to_string()
        });

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&source),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
            });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[layout],
                ..Default::default()
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&source),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.config.format,
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

        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Uniform Buffer {}", source)),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            source,
            rect,
            opacity,
        }
    }

    fn reload_shader(&mut self) {
        println!("Reloading shaders...");
        self.rebuild_components();
        self.background_component = self.create_component(
            &self.bind_group_layout,
            "src/starfield.wgsl".to_string(),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            1.0,
        );
        println!("All shaders reloaded.");
    }

    fn calculate_opacity(&mut self) -> f32 {
        if let Some(start) = self.fade_start_time {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed < 2.0 {
                // Fade out over 2 seconds (1.0 -> 0.0)
                1.0 - (elapsed / 2.0)
            } else if elapsed < 3.0 {
                // Hold black for 1 second
                0.0
            } else {
                // Reset
                self.fade_start_time = None;
                1.0
            }
        } else {
            1.0
        }
    }

    fn update(&mut self) {
        let global_uniforms = Uniforms {
            time: self.start_time.elapsed().as_secs_f32(),
            width: self.size.width as f32,
            height: self.size.height as f32,
            frame: self.frame_count,
            mouse_x: self.mouse_pos.0,
            mouse_y: self.mouse_pos.1,
            mouse_pressed: if self.mouse_pressed { 1 } else { 0 },
            opacity: 1.0,
        };

        let fade_factor = self.calculate_opacity();

        // Update background
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        // Prepare Egui
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

            // Draw Background
            rpass.set_viewport(0.0, 0.0, screen_w, screen_h, 0.0, 1.0);
            rpass.set_pipeline(&self.background_component.pipeline);
            rpass.set_bind_group(0, &self.background_component.bind_group, &[]);
            rpass.draw(0..3, 0..1);

            // Draw Components
            for comp in &self.components {
                let x = comp.rect.x * screen_w;
                let y = comp.rect.y * screen_h;
                let w = comp.rect.w * screen_w;
                let h = comp.rect.h * screen_h;

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
                    rpass.set_viewport(safe_x, safe_y, final_w, final_h, 0.0, 1.0);
                    rpass.set_pipeline(&comp.pipeline);
                    rpass.set_bind_group(0, &comp.bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                }
            }

            // Draw Egui
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

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Pick a Shader That Sparks Joy")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .unwrap(),
    );
    let mut state = pollster::block_on(State::new(window.clone()));

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => {
                let response = state.egui_state.on_window_event(&state.window, &event);
                if response.consumed {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(s) => {
                        state.size = s;
                        state.config.width = s.width;
                        state.config.height = s.height;
                        state.surface.configure(&state.device, &state.config);
                        state.rebuild_components();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        state.mouse_pos = (position.x as f32, position.y as f32);
                    }
                    WindowEvent::MouseInput {
                        state: s,
                        button: winit::event::MouseButton::Left,
                        ..
                    } => {
                        state.mouse_pressed = s == ElementState::Pressed;
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyR),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => state.reload_shader(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyF),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        state.fade_start_time = Some(Instant::now());
                    }
                    WindowEvent::RedrawRequested => {
                        state.update();
                        if let Err(e) = state.render() {
                            eprintln!("{:?}", e);
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        })
        .unwrap();
}
