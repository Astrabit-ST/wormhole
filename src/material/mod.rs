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
use crate::assets;
use crate::render;

use wgpu::util::DeviceExt;

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Debug, Clone, Copy)]
pub struct Data {
    base_color: render::Color,
    emissive: render::Color,
    metallic: f32,
    roughness: f32,
    flags: u32,
    _pad: [u8; 4],
}

bitflags::bitflags! {
    pub struct MaterialFlags: u32 {
        const HAS_BASE_COLOR_TEXTURE = 0b0000_0001;
        const HAS_METALLIC_ROUGHNESS_TEXTURE = 0b0000_0010;
        const HAS_EMISSIVE_TEXTURE = 0b0000_0100;
        const HAS_OCCLUSION_TEXTURE = 0b0000_1000;
        const HAS_NORMAL_MAP = 0b0001_0000;
    }
}

impl Material {
    pub fn from_gltf(
        render_state: &render::State,
        gltf_id: assets::GltfId,
        textures: &assets::Textures,
        material: gltf::Material<'_>,
    ) -> Self {
        let metallic_roughness = material.pbr_metallic_roughness();

        let mut flags = MaterialFlags::empty();

        let base_color = metallic_roughness.base_color_factor().into();
        let base_color_texture = metallic_roughness
            .base_color_texture()
            .map(|i| {
                flags |= MaterialFlags::HAS_BASE_COLOR_TEXTURE;
                textures.get_expect(assets::TextureId::Gltf(gltf_id, i.texture().index()))
            })
            .unwrap_or(textures.null_texture());

        let normal_texture = material
            .normal_texture()
            .map(|i| {
                flags |= MaterialFlags::HAS_NORMAL_MAP;
                textures.get_expect(assets::TextureId::Gltf(gltf_id, i.texture().index()))
            })
            .unwrap_or(textures.null_texture());

        let metallic = metallic_roughness.metallic_factor();
        let roughness = metallic_roughness.roughness_factor();
        let metallic_roughness_texture = metallic_roughness
            .metallic_roughness_texture()
            .map(|i| {
                flags |= MaterialFlags::HAS_METALLIC_ROUGHNESS_TEXTURE;
                textures.get_expect(assets::TextureId::Gltf(gltf_id, i.texture().index()))
            })
            .unwrap_or(textures.null_texture());

        let emissive: render::Color = material.emissive_factor().into();
        let emissive_texture = material
            .emissive_texture()
            .map(|i| {
                flags |= MaterialFlags::HAS_EMISSIVE_TEXTURE;
                textures.get_expect(assets::TextureId::Gltf(gltf_id, i.texture().index()))
            })
            .unwrap_or(textures.null_texture());

        let occlusion_texture = material
            .occlusion_texture()
            .map(|i| {
                flags |= MaterialFlags::HAS_OCCLUSION_TEXTURE;
                textures.get_expect(assets::TextureId::Gltf(gltf_id, i.texture().index()))
            })
            .unwrap_or(textures.null_texture());

        let flags = flags.bits();

        let material_buffer =
            render_state
                .wgpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("wormhole material data buffer"),
                    contents: bytemuck::bytes_of(&Data {
                        base_color,
                        metallic,
                        roughness,
                        emissive,
                        flags,
                        _pad: [0; 4],
                    }),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

        let bind_group = render_state
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wormhole material bind group"),
                layout: &render_state.bind_groups.material,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: material_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&base_color_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&base_color_texture.sampler),
                    },
                    // Normal
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                    },
                    // Metallic
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            &metallic_roughness_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(
                            &metallic_roughness_texture.sampler,
                        ),
                    },
                    // Emissive
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(&emissive_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(&emissive_texture.sampler),
                    },
                    // Occlusion
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::TextureView(&occlusion_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::Sampler(&occlusion_texture.sampler),
                    },
                ],
            });

        Self { bind_group }
    }
}

impl render::traits::Bindable for Material {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("wormhole material bind group layout"),
            entries: &[
                // Material data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Albedo
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Metal & Roughness
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Emissive
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Occlusion
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout {
        &render_state.bind_groups.material
    }
}
