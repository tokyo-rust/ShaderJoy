use std::borrow::Cow;
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent, ElementState, KeyEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{KeyCode, PhysicalKey},
};
use std::sync::Arc;
use std::fs;

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
    padding: f32, 
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipelines: Vec<wgpu::RenderPipeline>,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    start_time: Instant,
    frame_count: u32,
    mouse_pos: (f32, f32),
    mouse_pressed: bool,
    #[allow(dead_code)]
    window: Arc<winit::window::Window>,
}

impl State {
    async fn new(window: Arc<winit::window::Window>) -> State {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

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
            label: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            ..Default::default()
        });

        let shader_files = ["src/shader.wgsl", "src/twinkley.wgsl", "src/shader3.wgsl", "src/shader4.wgsl"];
        let mut pipelines = Vec::new();

        for file in shader_files {
            let shader_source = fs::read_to_string(file).unwrap_or_else(|_| include_str!("shader.wgsl").to_string());
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(file),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(file),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(config.format.into())],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            pipelines.push(pipeline);
        }

        Self {
            surface, device, queue, config, size,
            pipelines,
            uniform_buffer, bind_group, bind_group_layout,
            start_time: Instant::now(),
            frame_count: 0,
            mouse_pos: (0.0, 0.0),
            mouse_pressed: false,
            window,
        }
    }

    fn reload_shader(&mut self) {
        let shader_files = ["src/shader.wgsl", "src/twinkley.wgsl", "src/shader3.wgsl", "src/shader4.wgsl"];
        let mut new_pipelines = Vec::new();

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&self.bind_group_layout],
            ..Default::default()
        });

        for file in shader_files {
             // Fallback to shader.wgsl if read fails, or maybe just skip/log error. 
             // Using unwrap_or_else to match original behavior roughly, but here we fallback to shader.wgsl content for all.
             let shader_source = fs::read_to_string(file).unwrap_or_else(|_| {
                 println!("Failed to read {}", file);
                 include_str!("shader.wgsl").to_string()
             });
            
            let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(file),
                source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
            });

            let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(file),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(self.config.format.into())],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            new_pipelines.push(pipeline);
        }
        self.pipelines = new_pipelines;
        println!("All shaders reloaded.");
    }

    fn update(&mut self) {
        let uniforms = Uniforms {
            time: self.start_time.elapsed().as_secs_f32(),
            width: self.size.width as f32,
            height: self.size.height as f32,
            frame: self.frame_count,
            mouse_x: self.mouse_pos.0,
            mouse_y: self.mouse_pos.1,
            mouse_pressed: if self.mouse_pressed { 1 } else { 0 },
            padding: 0.0,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        self.frame_count += 1;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);

            let w = self.config.width as f32;
            let h = self.config.height as f32;
            let half_w = w / 2.0;
            let half_h = h / 2.0;

            // Define viewports for 2x2 grid
            // 0: Top-Left
            // 1: Top-Right
            // 2: Bottom-Left
            // 3: Bottom-Right
            // Note: WGPU coordinate system, (0,0) is usually top-left for viewports in most windowing systems, 
            // but let's assume standard behavior. If y grows down (which it usually does in screen coords), 
            // 0,0 is top-left.
            
            let viewports = [
                (0.0, 0.0, half_w, half_h),         // Top-Left
                (half_w, 0.0, half_w, half_h),      // Top-Right
                (0.0, half_h, half_w, half_h),      // Bottom-Left
                (half_w, half_h, half_w, half_h),   // Bottom-Right
            ];

            for (i, pipeline) in self.pipelines.iter().enumerate() {
                if i < 4 {
                    let (x, y, vw, vh) = viewports[i];
                    rpass.set_viewport(x, y, vw, vh, 0.0, 1.0);
                    rpass.set_pipeline(pipeline);
                    rpass.draw(0..3, 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_inner_size(winit::dpi::PhysicalSize::new(800, 600)).build(&event_loop).unwrap());
    let mut state = pollster::block_on(State::new(window.clone()));

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(s) => { state.size = s; state.config.width = s.width; state.config.height = s.height; state.surface.configure(&state.device, &state.config); }
            WindowEvent::CursorMoved { position, .. } => { state.mouse_pos = (position.x as f32, position.y as f32); }
            WindowEvent::MouseInput { state: s, button: winit::event::MouseButton::Left, .. } => { state.mouse_pressed = s == ElementState::Pressed; }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::KeyR), state: ElementState::Pressed, .. }, .. } => state.reload_shader(),
            WindowEvent::RedrawRequested => { state.update(); if let Err(e) = state.render() { eprintln!("{:?}", e); } }
            _ => {}
        },
        Event::AboutToWait => window.request_redraw(),
        _ => {}
    }).unwrap();
}
