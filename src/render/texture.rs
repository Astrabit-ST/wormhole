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

impl Texture {
    pub fn builder(image: &image::DynamicImage) -> TextureBuilder<'_> {
        TextureBuilder::new(image)
    }
}

pub struct TextureBuilder<'img> {
    image: &'img image::DynamicImage,
    format: wgpu::TextureFormat,
    filtering: wgpu::FilterMode,
    usage: wgpu::TextureUsages,
}

impl<'img> TextureBuilder<'img> {
    pub fn new(image: &'img image::DynamicImage) -> Self {
        Self {
            image,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            filtering: wgpu::FilterMode::Nearest,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    pub fn format(self, format: wgpu::TextureFormat) -> Self {
        Self { format, ..self }
    }

    pub fn filtering(self, filtering: wgpu::FilterMode) -> Self {
        Self { filtering, ..self }
    }

    pub fn usage(self, usage: wgpu::TextureUsages) -> Self {
        Self { usage, ..self }
    }

    pub fn build(self, render_state: &render::State) -> Texture {
        let image = self.image.to_rgba8();

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
                format: self.format,
                usage: self.usage,
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
                mag_filter: self.filtering,
                min_filter: self.filtering,
                mipmap_filter: self.filtering,
                ..Default::default()
            });

        Texture {
            texture,
            view,
            sampler,
        }
    }
}

impl Texture {}
