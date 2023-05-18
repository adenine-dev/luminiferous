use std::{
    mem::{self, size_of},
    num::NonZeroU32,
};

use bytemuck;
use wgpu::{util::DeviceExt, PushConstantRange, ShaderStages};

use shared::{
    glam::*,
    integrators::{Integrator, SimpleIntegrator},
    ShaderConstants,
};

use super::{Context, RenderOutput, RenderResult};
use crate::Config;

const SHADER: &[u8] = include_bytes!(env!("shaders.spv"));

unsafe fn to_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}

pub struct GpuContext {
    config: Config,
}

impl GpuContext {
    pub fn new(config: Config) -> Self {
        GpuContext { config }
    }
}

impl Context for GpuContext {
    fn render(&self) -> RenderResult {
        pollster::block_on((async || -> RenderResult {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok_or("could not get adapter :<")?;

            let (device, queue) = {
                let features = wgpu::Features::PUSH_CONSTANTS;
                let limits = wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                };
                adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            features,
                            limits,
                        },
                        None,
                    )
                    .await?
            };

            let texture_desc = wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
                view_formats: &vec![wgpu::TextureFormat::Rgba32Float],
            };
            let texture = device.create_texture(&texture_desc);
            let texture_view = texture.create_view(&Default::default());

            let pixel_size = wgpu::TextureFormat::Rgba32Float.describe().block_size as u32;

            let output_buffer_size =
                (pixel_size * self.config.width * self.config.height) as wgpu::BufferAddress;
            let output_buffer_desc = wgpu::BufferDescriptor {
                size: output_buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                label: None,
                mapped_at_creation: false,
            };
            let output_buffer = device.create_buffer(&output_buffer_desc);

            let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Vertex Shader"),
                source: wgpu::util::make_spirv(SHADER),
            });
            let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment Shader"),
                source: wgpu::util::make_spirv(SHADER),
            });

            let integrator_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            has_dynamic_offset: false,
                            min_binding_size: None,
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                        },
                    }],
                });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&integrator_bind_group_layout],
                    push_constant_ranges: &[PushConstantRange {
                        stages: ShaderStages::FRAGMENT,
                        range: 0..size_of::<ShaderConstants>() as u32,
                    }],
                });

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &vs_module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_desc.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
            });

            let integrator = Integrator::simple_integrator(vec2(
                self.config.width as f32,
                self.config.height as f32,
            ));

            // dbg!(unsafe { to_u8_slice(&integrator).len() });
            let integrator_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Integrator Buffer"),
                contents: unsafe { to_u8_slice(&integrator) },
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let integrator_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &integrator_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: integrator_buffer.as_entire_binding(),
                }],
            });

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let render_pass_desc = wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                };
                let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

                let shader_constants = ShaderConstants {
                    width: self.config.width,
                    height: self.config.height,
                };

                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_bind_group(0, &integrator_bind_group, &[]);
                render_pass.set_push_constants(
                    ShaderStages::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&shader_constants),
                );
                render_pass.draw(0..3, 0..1);
            }

            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: NonZeroU32::new(pixel_size * self.config.width),
                        rows_per_image: NonZeroU32::new(self.config.height),
                    },
                },
                texture_desc.size,
            );

            queue.submit(Some(encoder.finish()));

            let image_data = {
                let buffer_slice = output_buffer.slice(..);

                let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    tx.send(result).unwrap();
                });
                device.poll(wgpu::Maintain::Wait);
                rx.receive().await.ok_or("couldn't receive gpu stuffs")??;

                let data = buffer_slice.get_mapped_range();
                data.into_iter().copied().collect()
            };
            output_buffer.unmap();

            Ok(RenderOutput { image_data })
        })())
    }
}
