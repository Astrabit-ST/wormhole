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

pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl DepthTexture {
    pub fn builder() -> TextureBuilder {
        TextureBuilder::new()
    }

    pub fn resize(&mut self, render_state: &render::State) {
        let texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: render_state.surface_config.width,
                    height: render_state.surface_config.height,
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: self.texture.format(),
                usage: self.texture.usage(),
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.texture = texture;
        self.view = view;
    }
}

pub struct TextureBuilder {
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
}

impl TextureBuilder {
    pub fn new() -> Self {
        Self {
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    pub fn format(self, format: wgpu::TextureFormat) -> Self {
        Self { format, ..self }
    }

    pub fn usage(self, usage: wgpu::TextureUsages) -> Self {
        Self { usage, ..self }
    }

    pub fn build(self, render_state: &render::State) -> DepthTexture {
        let texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: render_state.surface_config.width,
                    height: render_state.surface_config.height,
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: self.format,
                usage: self.usage,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

        DepthTexture {
            texture,
            view,
            sampler,
        }
    }
}

impl DepthTexture {}
