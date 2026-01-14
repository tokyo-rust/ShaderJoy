//! Custom wgpu shader widget for Iced.
//!
//! Implements `iced::widget::shader::Program` and `Primitive` traits for
//! rendering WGSL fragment shaders in iced.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use iced::mouse::Cursor;
use iced::wgpu;
use iced::widget::shader::{self as iced_shader, Viewport};
use iced::Rectangle;
use shaderjoy_core::shaders::{DEFAULT_FRAGMENT_SHADER, VERTEX_SHADER};
use tracing::error;
use uuid::Uuid;

use shaderjoy_core::shaders::uniforms::ShaderUniforms;

pub fn shader_widget<Message>(data: ShaderData) -> iced::widget::Shader<Message, ShaderData>
where
    Message: 'static,
{
    iced::widget::shader(data)
}

/// Shader data for the shader widget.  This includes the shader code, uniforms,
/// selected status, and compilation errors.
#[derive(Debug, Clone)]
pub struct ShaderData {
    shader_data_inner: Arc<ShaderDataInner>,
}

impl ShaderData {
    pub fn new(wgsl_code: String, shader_code_id: Uuid) -> Self {
        Self {
            shader_data_inner: Arc::new(ShaderDataInner::new(wgsl_code, shader_code_id)),
        }
    }

    pub fn with_uniforms(mut self, uniforms: ShaderUniforms) -> Self {
        Arc::make_mut(&mut self.shader_data_inner).uniforms = uniforms;
        self
    }

    pub fn with_selected(mut self, is_selected: bool) -> Self {
        Arc::make_mut(&mut self.shader_data_inner).is_selected = is_selected;
        self
    }

    pub fn with_compilation_error(mut self, compilation_error: Option<String>) -> Self {
        Arc::make_mut(&mut self.shader_data_inner).compilation_error = compilation_error;
        self
    }
}

impl<Message> iced_shader::Program<Message> for ShaderData {
    type State = ();
    type Primitive = ShaderCellPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, _bounds: Rectangle) -> Self::Primitive {
        ShaderCellPrimitive::new(self.shader_data_inner.clone())
    }
}

#[derive(Debug, Clone)]
struct ShaderDataInner {
    wgsl_code: String,
    uniforms: ShaderUniforms,
    is_selected: bool,
    compilation_error: Option<String>,
    shader_code_id: Uuid,
}

impl ShaderDataInner {
    pub fn new(wgsl_code: String, shader_code_id: Uuid) -> Self {
        Self {
            wgsl_code,
            uniforms: ShaderUniforms::default(),
            is_selected: false,
            compilation_error: None,
            shader_code_id,
        }
    }
}

/// The primitive in the iced world for rendering a shader cell.
#[derive(Debug)]
pub struct ShaderCellPrimitive {
    shader_data: Arc<ShaderDataInner>,
}

impl ShaderCellPrimitive {
    fn new(shader_data: Arc<ShaderDataInner>) -> Self {
        Self { shader_data }
    }
}

impl iced_shader::Primitive for ShaderCellPrimitive {
    type Pipeline = ShaderCellPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        // Compile shader if needed and not empty
        if !self.shader_data.wgsl_code.is_empty() {
            if let Err(e) = pipeline.compile_shader_for_cache(
                &self.shader_data.wgsl_code,
                self.shader_data.shader_code_id,
            ) {
                error!(error = %e, "Failed to compile shader");
            }
        }

        let mut uniforms = self.shader_data.uniforms;
        uniforms.resolution = [
            viewport.physical_size().width as f32,
            viewport.physical_size().height as f32,
        ];
        queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let cell_pipeline = pipeline.get_pipeline_for_id(self.shader_data.shader_code_id);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ShaderCell render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&cell_pipeline);
        // TODO NOW can i make the box the entire viewport if i know the coordinates?
        pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// The wgpu pipeline data for drawing shader cells.  This includes a pipeline
/// cache for the different shaders, as well as all the shared uniform buffer,
/// bindgroups, and other wgpu stuff.
pub struct ShaderCellPipeline {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline_layout: wgpu::PipelineLayout,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    device: Arc<wgpu::Device>,
    // Cache pipelines per shader code id (wrapped in Arc to share without cloning)
    pipeline_cache: Arc<Mutex<HashMap<Uuid, Arc<wgpu::RenderPipeline>>>>,
    // Default pipeline for initial/fallback rendering
    default_pipeline: Arc<wgpu::RenderPipeline>,
}

impl ShaderCellPipeline {
    fn build_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        combined_shader: &str,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ShaderCell shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(combined_shader)),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ShaderCell render pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        })
    }

    fn compile_shader_for_cache(&self, wgsl_code: &str, shader_code_id: Uuid) -> Result<(), String> {
        let combined_shader = format!("{}\n{}", VERTEX_SHADER, wgsl_code);

        let mut cache = self.pipeline_cache.lock().unwrap();

        // Already cached by another cell?
        if cache.contains_key(&shader_code_id) {
            return Ok(());
        }

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::build_pipeline(
                &self.device,
                &self.pipeline_layout,
                &combined_shader,
                self.format,
            )
        })) {
            Ok(pipeline) => {
                cache.insert(shader_code_id, Arc::new(pipeline));
                Ok(())
            }
            Err(_) => {
                error!("Shader compilation panicked");
                Err("Shader compilation failed".to_string())
            }
        }
    }

    fn get_pipeline_for_id(&self, shader_code_id: Uuid) -> Arc<wgpu::RenderPipeline> {
        self.pipeline_cache
            .lock()
            .unwrap()
            .get(&shader_code_id)
            .cloned()
            .unwrap_or_else(|| Arc::clone(&self.default_pipeline))
    }
}

impl iced_shader::Pipeline for ShaderCellPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let device = Arc::new(device.clone());
        let default_shader = DEFAULT_FRAGMENT_SHADER;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ShaderCell uniforms"),
            size: std::mem::size_of::<ShaderUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ShaderCell bind group layout"),
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
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ShaderCell bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ShaderCell pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let combined_shader = format!("{}\n{}", VERTEX_SHADER, default_shader);
        let default_pipeline = Arc::new(Self::build_pipeline(
            &device,
            &pipeline_layout,
            &combined_shader,
            format,
        ));

        Self {
            uniform_buffer,
            bind_group,
            pipeline_layout,
            bind_group_layout,
            format,
            device,
            pipeline_cache: Arc::new(Mutex::new(HashMap::new())),
            default_pipeline,
        }
    }
}
