// Copyright (C) 2023 Lily Lyons
//
// This file is part of wormhole.
//
// wormhole is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// wormhole is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with wormhole.  If not, see <http://www.gnu.org/licenses/>.
use crate::render;

pub struct Buffer {
    // Alpha channel contains material roughness
    pub color_roughness: render::Texture,
    // Alpha channel contains material metallicity
    pub normal_metallicity: render::Texture,
    // Alpha channel contains occlusion
    pub position_occlusion: render::Texture,
    pub emissive: render::Texture,

    pub depth: render::Texture,

    pub bind_group: wgpu::BindGroup, // FIXME: streamline
}

impl Buffer {
    pub fn new(render_state: &render::State) -> Self {
        let color_roughness =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let normal_metallicity =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let position_occlusion =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let emissive =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);

        let depth = render::Texture::new_screen_size(render_state, render::TextureFormat::DEPTH);

        let bind_group = render_state
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wormhole gbuffer bind group"),
                layout: &render_state.bind_groups.gbuffer,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&color_roughness.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&color_roughness.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&normal_metallicity.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&normal_metallicity.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&position_occlusion.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&position_occlusion.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&emissive.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&emissive.sampler),
                    },
                ],
            });

        Self {
            color_roughness,
            normal_metallicity,
            position_occlusion,
            emissive,

            depth,

            bind_group,
        }
    }

    pub fn resize_to_screen(&mut self, render_state: &render::State) {
        self.color_roughness.resize_to_screen(render_state);
        self.normal_metallicity.resize_to_screen(render_state);
        self.position_occlusion.resize_to_screen(render_state);
        self.emissive.resize_to_screen(render_state);

        self.bind_group = render_state
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wormhole gbuffer bind group"),
                layout: &render_state.bind_groups.gbuffer,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.color_roughness.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.color_roughness.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&self.normal_metallicity.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.normal_metallicity.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&self.position_occlusion.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&self.position_occlusion.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&self.emissive.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&self.emissive.sampler),
                    },
                ],
            });

        self.depth.resize_to_screen(render_state);
    }

    pub fn as_color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 4] {
        [
            Some(wgpu::RenderPassColorAttachment {
                view: &self.color_roughness.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal_metallicity.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.position_occlusion.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.emissive.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }),
        ]
    }

    pub fn depth_stencil_attachment_initial(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }
    }

    pub fn depth_stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            }),
            stencil_ops: None,
        }
    }
}

impl render::traits::Bindable for Buffer {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("wormhole gbuffer bind group layout"),
            entries: &[
                // Color & Roughness
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
                // Normal & Metallicity
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Position & Occlusion
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Emissive
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        };

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout {
        &render_state.bind_groups.gbuffer
    }
}
