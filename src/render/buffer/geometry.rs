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
    pub sampler: wgpu::Sampler,
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
        let sampler = render_state
            .wgpu
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("gbuffer sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
        let color_roughness =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let normal_metallicity =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let position_occlusion =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let emissive =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);

        let depth = render::Texture::new_screen_size(render_state, render::TextureFormat::DEPTH);

        let bind_group = render::BindGroupBuilder::new()
            .append_sampler(&sampler)
            .append_texture_view(&color_roughness.view)
            .append_texture_view(&normal_metallicity.view)
            .append_texture_view(&position_occlusion.view)
            .append_texture_view(&emissive.view)
            .build(
                &render_state.wgpu.device,
                Some("wormhole gbuffer bind group"),
                &render_state.bind_groups.gbuffer,
            );

        Self {
            sampler,
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

        self.bind_group = render::BindGroupBuilder::new()
            .append_sampler(&self.sampler)
            .append_texture_view(&self.color_roughness.view)
            .append_texture_view(&self.normal_metallicity.view)
            .append_texture_view(&self.position_occlusion.view)
            .append_texture_view(&self.emissive.view)
            .build(
                &render_state.wgpu.device,
                Some("wormhole gbuffer bind group"),
                &render_state.bind_groups.gbuffer,
            );

        self.depth.resize_to_screen(render_state);
    }

    pub fn as_color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 4] {
        [
            Some(wgpu::RenderPassColorAttachment {
                view: &self.color_roughness.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal_metallicity.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.position_occlusion.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.emissive.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
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
