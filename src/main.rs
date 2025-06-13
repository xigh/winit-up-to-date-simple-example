use anyhow::{anyhow, Result};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

// const WINDOW_H: u32 = 240; // 480; // 360;
// const WINDOW_W: u32 = 320; // 640;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    last_render_time: std::time::Instant,
    sum_render_time: std::time::Duration,
    sum_render_count: u32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    palette_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    frame: u32,
    width: u32,
    height: u32,
}

impl State {
    async fn new(window: Arc<Window>, width: u32, height: u32) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture View"),
            format: Some(wgpu::TextureFormat::R8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let palette_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Palette Texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Initialize default palette
        let mut default_palette = [[0u8; 4]; 256];
        for i in 0..256 {
            default_palette[i] = [
                (i * 2) as u8, // R
                (i * 3) as u8, // G
                (i * 4) as u8, // B
                255,           // A
            ];
        }

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Palette Staging Buffer"),
            size: 256 * 4,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });

        staging_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(&default_palette));
        staging_buffer.unmap();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Palette Update Encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256 * 4),
                    rows_per_image: Some(1),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &palette_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));

        let palette_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&palette_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("Palette View"),
                        format: Some(wgpu::TextureFormat::Rgba8Unorm),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: Some(1),
                        base_array_layer: 0,
                        array_layer_count: Some(1),
                        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                    })),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&palette_sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            last_render_time: std::time::Instant::now(),
            sum_render_time: std::time::Duration::from_secs(0),
            sum_render_count: 0,
            vertex_buffer,
            index_buffer,
            texture,
            palette_texture,
            bind_group,
            render_pipeline,
            frame: 0,
            width,
            height,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we're going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface.configure(
                &self.device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: self.surface_format,
                    width: new_size.width,
                    height: new_size.height,
                    present_mode: wgpu::PresentMode::Fifo,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            // Update vertex buffer with new size
            let (new_vertex_buffer, _) = Self::create_vertex_buffer(&self, &self.device, new_size);
            self.vertex_buffer = new_vertex_buffer;
        }
    }

    fn create_vertex_buffer(
        &self,
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> (wgpu::Buffer, [Vertex; 4]) {
        // Calculate aspect ratio and zoom factor
        let width_ratio = size.width as f32 / self.width as f32;
        let height_ratio = size.height as f32 / self.height as f32;
        let zoom_factor = width_ratio.min(height_ratio).floor();

        // Calculate scaled dimensions
        let scaled_width = self.width as f32 * zoom_factor;
        let scaled_height = self.height as f32 * zoom_factor;

        // Create vertices with scaled dimensions
        let vertices = [
            Vertex {
                position: [
                    -scaled_width / size.width as f32,
                    -scaled_height / size.height as f32,
                ],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [
                    scaled_width / size.width as f32,
                    -scaled_height / size.height as f32,
                ],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [
                    scaled_width / size.width as f32,
                    scaled_height / size.height as f32,
                ],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [
                    -scaled_width / size.width as f32,
                    scaled_height / size.height as f32,
                ],
                tex_coords: [0.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        (vertex_buffer, vertices)
    }

    #[allow(dead_code)]
    fn create_cycling_palette(&self, time: f32) -> [[u8; 4]; 256] {
        let mut palette = [[0u8; 4]; 256];
        let x = (256 as f32 * time) as u8;
        // println!("x: {} time: {}", x, time);
        for i in 0..256 {
            let r = x.wrapping_add((i * 2) as u8);
            let g = x.wrapping_add((i * 4) as u8);
            let b = x.wrapping_add(i as u8);
            // let hue = (i as f32 / 256.0 + time * 0.1) % 1.0;
            // Convertir HSV en RGB (simplifié)
            // let r = ((hue * 6.0).sin().abs() * 255.0) as u8;
            // let g = (((hue * 6.0 + 2.0) * std::f32::consts::PI / 3.0).sin().abs() * 255.0) as u8;
            // let b = (((hue * 6.0 + 4.0) * std::f32::consts::PI / 3.0).sin().abs() * 255.0) as u8;
            palette[i as usize] = [r, g, b, 255];
        }
        palette
    }

    pub fn render(&mut self, data: &[u8], palette: &[[u8; 4]; 256]) -> Result<(), wgpu::SurfaceError> {
        let now = std::time::Instant::now();
        let dur = now - self.last_render_time;
        self.sum_render_time += dur;
        self.sum_render_count += 1;
        self.last_render_time = now;

        if self.sum_render_count > 100 {
            println!(
                "render time: {:?} fps: {} [{}x{}]",
                self.sum_render_time / self.sum_render_count,
                self.sum_render_count as f64 / self.sum_render_time.as_secs_f64(),
                self.size.width,
                self.size.height,
            );
            self.sum_render_time = std::time::Duration::from_secs(0);
            self.sum_render_count = 0;
        }

        // Update palette with cycling colors
        self.frame += 1;
        // let palette = self.create_cycling_palette((self.frame % 60) as f32 / 60.0);
        self.update_palette(palette);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Update texture with emulator data
        let bytes_per_row = ((self.width + 255) & !255) as u32;  // Align to 256 bytes (wgpu's texture alignment)
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (bytes_per_row * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });

        // Copy data with padding
        let mut buffer = vec![0u8; (bytes_per_row * self.height) as usize];
        for y in 0..self.height {
            let src_start = (y * self.width) as usize;
            let dst_start = (y * bytes_per_row) as usize;
            buffer[dst_start..dst_start + self.width as usize].copy_from_slice(&data[src_start..src_start + self.width as usize]);
        }

        staging_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(&buffer);
        staging_buffer.unmap();

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update_palette(&mut self, palette: &[[u8; 4]; 256]) {
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Palette Staging Buffer"),
            size: 256 * 4,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });

        staging_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(palette));
        staging_buffer.unmap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Palette Update Encoder"),
            });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256 * 4),
                    rows_per_image: Some(1),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.palette_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

#[derive(Clone)]
struct Rgba (u8, u8, u8, u8);

impl Rgba {
    fn to_array(&self) -> [u8; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl PartialEq for Rgba {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2 && self.3 == other.3
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self(0, 0, 0, 0)
    }
}

#[derive(Default)]
struct App {
    windows: Vec<State>, // space for future use, by example surface texture
    width: u32,
    height: u32,
    data: Vec<u8>,
    palette: Vec<Rgba>,
}

impl App {
    pub fn new(width: u32, height: u32, data: Vec<u8>, palette: Vec<Rgba>) -> Self {
        Self {
            windows: Vec::new(),
            width,
            height,
            data,
            palette,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        let window_attr = Window::default_attributes()
            .with_title("Window")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(window_attr).unwrap());
        let state = pollster::block_on(State::new(window, self.width, self.height));
        self.windows.push(state);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        println!("suspended");
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("exiting");
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // todo ???
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\tCloseRequested");
                self.windows.retain(|state| state.get_window().id() != id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    println!(
                        "\tKeyboardInput {:?} - {:?} - {}",
                        code, event.state, is_synthetic
                    );
                    if !is_synthetic {
                        // windows[idx].on_key_input(code, event.state == ElementState::Pressed, &q);
                    }
                    match code {
                        KeyCode::Escape => {
                            self.windows.retain(|state| state.get_window().id() != id);
                            if self.windows.is_empty() {
                                event_loop.exit();
                            }
                        }
                        KeyCode::KeyF => {
                            println!("KeyF");
                            // toggle fullscreen
                            if let Some(state) = self
                                .windows
                                .iter_mut()
                                .find(|state| state.get_window().id() == id)
                            {
                                state.get_window().set_fullscreen(
                                    if state.get_window().fullscreen().is_some() {
                                        None
                                    } else {
                                        Some(Fullscreen::Borderless(None))
                                    },
                                );
                            }
                        }
                        _ => {
                            // println!("Other");
                        }
                    }
                }
                _ => {}
            },
            // WindowEvent::ActivationTokenDone { serial, token } => todo!(),
            WindowEvent::Resized(new_size) => {
                println!("\tResized {:?}", new_size);
                if let Some(state) = self
                    .windows
                    .iter_mut()
                    .find(|state| state.get_window().id() == id)
                {
                    state.resize(new_size);
                }
            }
            // WindowEvent::Moved(_) => todo!(),
            WindowEvent::Destroyed => {
                println!("\tDestroyed");
            }
            // WindowEvent::DroppedFile(_) => todo!(),
            // WindowEvent::HoveredFile(_) => todo!(),
            // WindowEvent::HoveredFileCancelled => todo!(),
            WindowEvent::Focused(focused) => {
                println!("\tFocused {}", focused);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                println!("\tModifiersChanged {:?}", modifiers);
            }
            // WindowEvent::Ime(_) => todo!(),
            // WindowEvent::CursorMoved { device_id, position } => todo!(),
            // WindowEvent::CursorEntered { device_id } => todo!(),
            // WindowEvent::CursorLeft { device_id } => todo!(),
            WindowEvent::MouseWheel { delta, .. } => {
                println!("\tMouseWheel {:?}", delta);
            }
            WindowEvent::MouseInput { button, .. } => {
                println!("\tMouseInput {:?}", button);
            }
            // WindowEvent::TouchpadMagnify { device_id, delta, phase } => todo!(),
            // WindowEvent::SmartMagnify { device_id } => todo!(),
            // WindowEvent::TouchpadRotate { device_id, delta, phase } => todo!(),
            // WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
            // WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
            // WindowEvent::Touch(_) => todo!(),
            // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => todo!(),
            // WindowEvent::ThemeChanged(_) => todo!(),
            WindowEvent::Occluded(occluded) => {
                // not raised on Windows 11
                println!("\tOccluded {}", occluded);
            }
            WindowEvent::RedrawRequested => {
                // println!("\tRedrawRequested");
                if let Some(state) = self
                    .windows
                    .iter_mut()
                    .find(|state| state.get_window().id() == id)
                {
                    // source palette might contains fewer than 256 colors, so we need to pad it
                    let mut full_palette = [[0u8; 4]; 256];
                    // println!("Palette length: {}", self.palette.len());
                    for (i, color) in self.palette.iter().enumerate() {
                        full_palette[i] = color.to_array();
                        // println!("Palette[{}]: {:?}", i, color.to_array());
                    }
                    // println!("Data length: {}", self.data.len());
                    // println!("First 10 data values: {:?}", &self.data[..10.min(self.data.len())]);
                    state.render(&self.data, &full_palette).unwrap();
                    state.get_window().request_redraw(); // Request next frame
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let tracker_png = include_bytes!("../tracker.png");
    let tracker_image = image::load_from_memory(tracker_png).unwrap();
    let tracker_rgba = tracker_image.to_rgba8();
    let tracker_width = tracker_rgba.width();
    let tracker_height = tracker_rgba.height();
    let mut palette = Vec::new();
    let mut tracker_data = Vec::new();
    for y in 0..tracker_height {
        for x in 0..tracker_width {
            let pixel = tracker_rgba.get_pixel(x, y);
            let pixel = Rgba(pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3]);
            let idx = {
                let mut found_idx = None;
                for i in 0..palette.len() {
                    if pixel == palette[i] {
                        found_idx = Some(i);
                        break;
                    }
                }
                if let Some(i) = found_idx {
                    i as u8
                } else {
                    palette.push(pixel);
                    (palette.len() - 1) as u8
                }
            };
            tracker_data.push(idx);
        }
    }
    // let tracker_rgba_data = tracker_rgba.into_raw();
    println!("tracker_width: {}", tracker_width);
    println!("tracker_height: {}", tracker_height);
    println!("tracker_data: {:?}", tracker_data.len());
    println!("palette: {:?}", palette.len());

    let event_loop = EventLoop::new()?;

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(tracker_width, tracker_height, tracker_data, palette);
    event_loop.run_app(&mut app).map_err(|err| anyhow!(err))
}
