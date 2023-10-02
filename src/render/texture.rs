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
    pub format: wgpu::TextureFormat,
    pub filtering: wgpu::FilterMode,
    pub usage: wgpu::TextureUsages,
    pub compare: Option<wgpu::CompareFunction>,
}

impl TextureFormat {
    pub const GENERIC: Self = TextureFormat {
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::COPY_SRC
            .union(wgpu::TextureUsages::COPY_DST)
            .union(wgpu::TextureUsages::TEXTURE_BINDING),
        compare: None,
    };

    pub const NORMAL: Self = TextureFormat {
        format: wgpu::TextureFormat::Rgba8Unorm,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::COPY_SRC
            .union(wgpu::TextureUsages::COPY_DST)
            .union(wgpu::TextureUsages::TEXTURE_BINDING),
        compare: None,
    };

    pub const DEPTH: Self = TextureFormat {
        format: wgpu::TextureFormat::Depth32Float,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::RENDER_ATTACHMENT),
        compare: Some(wgpu::CompareFunction::LessEqual),
    };

    pub const GBUFFER: Self = TextureFormat {
        format: wgpu::TextureFormat::Rgba16Float,
        filtering: wgpu::FilterMode::Nearest,
        usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::RENDER_ATTACHMENT),
        compare: None,
    };
}

impl Texture {
    pub fn new(render_state: &render::State, size: wgpu::Extent3d, format: TextureFormat) -> Self {
        let texture = render_state
            .wgpu
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
            .wgpu
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: format.filtering,
                min_filter: format.filtering,
                mipmap_filter: format.filtering,
                compare: format.compare,
                ..Default::default()
            });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn new_screen_size(render_state: &render::State, format: TextureFormat) -> Self {
        Self::new(
            render_state,
            wgpu::Extent3d {
                width: render_state.wgpu.surface_config.width,
                height: render_state.wgpu.surface_config.height,
                depth_or_array_layers: 1,
            },
            format,
        )
    }

    pub fn from_bytes(
        render_state: &render::State,
        image: &[u8],
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Self {
        let texture = render_state.wgpu.device.create_texture_with_data(
            &render_state.wgpu.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                format: format.format,
                usage: format.usage,
                view_formats: &[],
            },
            image,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_state
            .wgpu
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
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
        Self::from_bytes(render_state, &image, image.width(), image.height(), format)
    }

    pub fn resize_to_screen(&mut self, render_state: &render::State) {
        let texture = render_state
            .wgpu
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: render_state.wgpu.surface_config.width,
                    height: render_state.wgpu.surface_config.height,
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
