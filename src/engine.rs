use crate::app::{self};
use anyhow::{Context, Result};
use pollster::FutureExt;
use std::sync::Arc;
use winit::{
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
}

impl<'a> app::Application for Engine<'a> {
    fn new_app(window: Arc<app::Window>) -> Self {
        let renderer = Renderer::new(window).unwrap();

        Self { renderer }
    }

    fn handle_event(&mut self, event: &app::AppEvent) -> app::AppSignal {
        use app::AppEvent;
        use app::AppSignal;
        use app::WindowEvent;

        match event {
            AppEvent::Window(
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::F12),
                            ..
                        },
                    ..
                },
                ..,
            ) => AppSignal::Quit,
            _ => AppSignal::Continue,
        }
    }

    fn update(&mut self) -> app::AppSignal {

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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
        })
    }
}
