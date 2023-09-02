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
use wgpu::util::DeviceExt;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub struct TextureFormat {
    format: wgpu::TextureFormat,
    filtering: wgpu::FilterMode,
    usage: wgpu::TextureUsages,
}

impl TextureFormat {
    pub const GENERIC: Self = TextureFormat {
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::COPY_SRC
            .union(wgpu::TextureUsages::COPY_DST)
            .union(wgpu::TextureUsages::TEXTURE_BINDING),
    };

    pub const NORMAL: Self = TextureFormat {
        format: wgpu::TextureFormat::Rgba8Unorm,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::COPY_SRC
            .union(wgpu::TextureUsages::COPY_DST)
            .union(wgpu::TextureUsages::TEXTURE_BINDING),
    };
}

impl Texture {
    pub fn new(render_state: &render::State, size: wgpu::Extent3d, format: TextureFormat) -> Self {
        let texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: format.format,
                usage: format.usage,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: format.filtering,
                min_filter: format.filtering,
                mipmap_filter: format.filtering,
                ..Default::default()
            });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn from_image(
        render_state: &render::State,
        image: &image::DynamicImage,
        format: TextureFormat,
    ) -> Self {
        let image = image.to_rgba8();
        let texture = render_state.device.create_texture_with_data(
            &render_state.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: format.format,
                usage: format.usage,
                view_formats: &[],
            },
            &image,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: format.filtering,
                min_filter: format.filtering,
                mipmap_filter: format.filtering,
                ..Default::default()
            });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
