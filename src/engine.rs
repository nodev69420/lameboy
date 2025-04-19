use crate::app::{self};
use anyhow::{Context, Result};
use cgmath::{Matrix4, SquareMatrix};
use image::GenericImageView;
use pollster::FutureExt;
use std::{path::Path, sync::Arc};
use wgpu::{util::DeviceExt, Color};
use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Engine<'a> {
    time: Time,
    renderer: Renderer<'a>,
}

impl<'a> app::Application for Engine<'a> {
    fn new_app(window: Arc<app::Window>) -> Self {
        let renderer = Renderer::new(window).unwrap();
        let time = Time::start();

        Self { time, renderer }
    }

    fn handle_event(&mut self, event: &app::AppEvent) -> app::AppSignal {
        use app::AppEvent;
        use app::AppSignal;
        use app::WindowEvent;

        match event {
            AppEvent::Window(window_event) => match window_event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::F12),
                            ..
                        },
                    ..
                } => AppSignal::Quit,

                WindowEvent::Resized(new_size) => {
                    self.renderer.resize(*new_size);
                    AppSignal::Continue
                }

                _ => AppSignal::Continue,
            },
            _ => AppSignal::Continue,
        }
    }

    fn update(&mut self) -> app::AppSignal {
        self.time = self.time.next();
        self.renderer.draw();

        // println!("{:#?}", self.time);

        use app::AppSignal;
        AppSignal::Continue
    }
}

struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    matrix_buffer: wgpu::Buffer,
    matrix_bind_group: wgpu::BindGroup,

    pipeline: wgpu::RenderPipeline,
    texture: Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Arc<app::Window>) -> Result<Self> {
        let size = window.inner_size();

        assert!(size.width != 0 || size.height != 0);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .context("renderer failed to create wgpu::Surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .context("renderer failed to create wgpu::Adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .block_on()
            .context(
                "renderer failed to create wgpu::Device and wgpu::Queue",
            )?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = surface_caps
            .present_modes
            .iter()
            .find(|f| **f == wgpu::PresentMode::Mailbox)
            .copied()
            .unwrap_or(surface_caps.present_modes[0]);

        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .find(|mode| **mode == wgpu::CompositeAlphaMode::Inherit)
            .copied()
            .unwrap_or(surface_caps.alpha_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // matrix layout

        let matrix_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Matrix Buffer"),
                contents: bytemuck::cast_slice(&[MatrixUniform::from(
                    Matrix4::identity(),
                )]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            });

        let matrix_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Matrix Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let matrix_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Matrix Bind Group"),
                layout: &matrix_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: matrix_buffer.as_entire_binding(),
                }],
            });

        // texture layout

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        // pipeline

        let pipeline = make_render_pipeline(
            &device,
            "Gameboy Texture",
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
            config.format,
            "./ass/shaders/texture.wgsl",
            &[&matrix_bind_group_layout, &texture_bind_group_layout],
            &[],
        );

        // texture

        // let texture =
        //     load_texture(&device, &queue, "Example", "./ass/test.png");
        let mut canvas = Canvas::new(256, 256, Pixel::BLUE);
        for pixel in canvas.pixels.iter_mut() {
            *pixel = rand::random::<u32>().into();
        }
        let texture = canvas.gpu_load("CUM", &device, &queue);

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Example"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            &texture.sampler,
                        ),
                    },
                ],
            });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,

            matrix_buffer,
            matrix_bind_group,

            pipeline,

            texture,
            texture_bind_group_layout,
            texture_bind_group,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if !new_size.eq(&self.size) {
            assert!(new_size.width != 0 && new_size.height != 0);

            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn draw(&mut self) {
        //
        self.queue.write_buffer(
            &self.matrix_buffer,
            0,
            bytemuck::cast_slice(&[MatrixUniform::from(make_texture_matrix(
                self.config.width,
                self.config.height,
                2.0,
            ))]),
        );

        //
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            },
        );

        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.matrix_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }

    pub fn use_internals(&self) -> (&wgpu::Device, &wgpu::Queue) {
        (&self.device, &self.queue)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub const WHITE: Pixel = Pixel {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const RED: Pixel = Pixel {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    pub const GREEN: Pixel = Pixel {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    pub const BLUE: Pixel = Pixel {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
}

impl Into<u32> for Pixel {
    fn into(self) -> u32 {
        ((self.r as u32) & 0x00_00_00_FF)
            | (((self.g as u32) << 8) & 0x00_00_FF_00)
            | (((self.b as u32) << 16) & 0x00_FF_00_00)
            | (((self.a as u32) << 24) & 0xFF_00_00_00)
    }
}

impl From<u32> for Pixel {
    fn from(value: u32) -> Self {
        Self {
            r: (value & 0x00_00_00_FF) as u8,
            g: ((value & 0x00_00_FF_00) >> 8) as u8,
            b: ((value & 0x00_FF_00_00) >> 16) as u8,
            a: ((value & 0xFF_00_00_00) >> 24) as u8,
        }
    }
}

pub struct Canvas {
    pub pixels: Box<[Pixel]>,
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32, fill: Pixel) -> Self {
        assert!(width != 0 && height != 0);
        let area = width * height;

        let pixels = vec![fill; area as usize].into_boxed_slice();

        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn gpu_load(
        &self,
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Texture {
        Texture::from_data(
            device,
            queue,
            name,
            bytemuck::cast_slice(&self.pixels),
            self.width,
            self.height,
        )
    }
}

fn make_texture_matrix(width: u32, height: u32, scale: f32) -> Matrix4<f32> {
    assert!(width != 0 && height != 0);
    let screen =
        cgmath::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
    let scale = 256.0 * scale;
    screen * Matrix4::from_nonuniform_scale(scale, scale, 1.0)
}

fn make_render_pipeline<F>(
    device: &wgpu::Device,
    name: &str,
    topology: wgpu::PrimitiveTopology,
    polygon_mode: wgpu::PolygonMode,
    format: wgpu::TextureFormat,
    shader_path: F,
    bind_groups: &[&wgpu::BindGroupLayout],
    descriptors: &[wgpu::VertexBufferLayout],
) -> wgpu::RenderPipeline
where
    F: AsRef<Path>,
{
    let shader = load_shader(device, shader_path);

    let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&["Render Pipeline Layout: ", name].concat()),
            // bind groups
            bind_group_layouts: bind_groups,
            push_constant_ranges: &[],
        });

    let render_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&["Render Pipeline: ", name].concat()),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // descriptors
                buffers: descriptors,
                compilation_options: wgpu::PipelineCompilationOptions::default(
                ),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(
                ),
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode,
                unclipped_depth: false,
                conservative: false,
            },

            // depth
            depth_stencil: None,

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

    render_pipeline
}

fn load_shader<F>(device: &wgpu::Device, path: F) -> wgpu::ShaderModule
where
    F: AsRef<Path>,
{
    let shader_source = std::fs::read(path).unwrap();
    let shader_utf8 = std::str::from_utf8(&shader_source).unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(shader_utf8)),
    });

    shader
}

fn load_texture<F>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    name: &str,
    path: F,
) -> Texture
where
    F: AsRef<Path>,
{
    let image_bytes = std::fs::read(&path).unwrap();
    let image_loaded = image::load_from_memory(&image_bytes).unwrap();
    let image_rgba = image_loaded.flipv().to_rgba8();
    let (width, height) = image_loaded.dimensions();

    Texture::from_data(device, queue, name, &image_rgba, width, height)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MatrixUniform {
    pub data: [[f32; 4]; 4],
}

impl From<Matrix4<f32>> for MatrixUniform {
    fn from(value: Matrix4<f32>) -> Self {
        Self { data: value.into() }
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub curr: std::time::SystemTime,
    pub delta: f32,
    pub runtime: f32,
    pub fps: usize,
}

impl Time {
    pub fn start() -> Self {
        Self {
            curr: std::time::SystemTime::now(),
            delta: 0.0,
            runtime: 0.0,
            fps: 0,
        }
    }

    pub fn next(&self) -> Time {
        let elapsed = self.curr.elapsed().expect(
            "Time::next failed, this could only occur during a y2k style bug!",
        );
        let delta = elapsed.as_secs_f32();

        Self {
            delta,
            runtime: self.runtime + delta,
            fps: (1.0 / delta) as usize,
            curr: std::time::SystemTime::now(),
        }
    }

    pub fn make_seed(&self) -> u64 {
        self.curr
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}
