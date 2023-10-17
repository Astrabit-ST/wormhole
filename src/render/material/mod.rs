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

pub struct Material {
    pub base_color: render::Color,
    pub base_color_texture: Option<assets::TextureId>,

    pub emissive: render::Color,
    pub emissive_texture: Option<assets::TextureId>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<assets::TextureId>,

    pub normal_texture: Option<assets::TextureId>,
    pub occlusion_texture: Option<assets::TextureId>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    pub base_color: render::Color,
    pub base_color_texture: u32,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: u32,

    pub emissive: render::Color,
    pub emissive_texture: u32,

    pub normal_texture: u32,
    pub occlusion_texture: u32,
    pub flags: MaterialFlags,
}

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    #[derive(bytemuck::Pod, bytemuck::Zeroable)]
    pub struct MaterialFlags: u32 {
        const HAS_BASE_COLOR_TEXTURE = 0b0000_0001;
        const HAS_METALLIC_ROUGHNESS_TEXTURE = 0b0000_0010;
        const HAS_EMISSIVE_TEXTURE = 0b0000_0100;
        const HAS_OCCLUSION_TEXTURE = 0b0000_1000;
        const HAS_NORMAL_TEXTURE = 0b0001_0000;
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: render::Color::from([1.0; 3]),
            base_color_texture: None,

            emissive: render::Color::default(),
            emissive_texture: None,

            metallic: 0.5,
            roughness: 0.5,
            metallic_roughness_texture: None,

            normal_texture: None,
            occlusion_texture: None,
        }
    }
}

impl Material {
    pub fn from_gltf(gltf_id: assets::GltfId, material: gltf::Material<'_>) -> Self {
        let metallic_roughness = material.pbr_metallic_roughness();

        let mut flags = MaterialFlags::empty();

        let base_color = metallic_roughness.base_color_factor().into();
        let base_color_texture = metallic_roughness.base_color_texture().map(|i| {
            flags |= MaterialFlags::HAS_BASE_COLOR_TEXTURE;
            assets::TextureId::Gltf(gltf_id, i.texture().index())
        });

        let normal_texture = material.normal_texture().map(|i| {
            flags |= MaterialFlags::HAS_NORMAL_TEXTURE;
            assets::TextureId::Gltf(gltf_id, i.texture().index())
        });

        let metallic = metallic_roughness.metallic_factor();
        let roughness = metallic_roughness.roughness_factor();
        let metallic_roughness_texture = metallic_roughness.metallic_roughness_texture().map(|i| {
            flags |= MaterialFlags::HAS_METALLIC_ROUGHNESS_TEXTURE;
            assets::TextureId::Gltf(gltf_id, i.texture().index())
        });

        let emissive: render::Color = material.emissive_factor().into();
        let emissive_texture = material.emissive_texture().map(|i| {
            flags |= MaterialFlags::HAS_EMISSIVE_TEXTURE;
            assets::TextureId::Gltf(gltf_id, i.texture().index())
        });

        let occlusion_texture = material.occlusion_texture().map(|i| {
            flags |= MaterialFlags::HAS_OCCLUSION_TEXTURE;
            assets::TextureId::Gltf(gltf_id, i.texture().index())
        });

        Self {
            base_color,
            base_color_texture,

            emissive,
            emissive_texture,

            metallic,
            roughness,
            metallic_roughness_texture,

            normal_texture,
            occlusion_texture,
        }
    }

    pub fn as_data(&self, textures: &assets::Textures) -> Data {
        Data {
            base_color: self.base_color,
            base_color_texture: self
                .base_color_texture
                .and_then(|i| textures.id_to_bindgroup_index(i))
                .unwrap_or_default() as u32,

            metallic: self.metallic,
            roughness: self.roughness,
            metallic_roughness_texture: self
                .metallic_roughness_texture
                .and_then(|i| textures.id_to_bindgroup_index(i))
                .unwrap_or_default() as u32,

            emissive: self.emissive,
            emissive_texture: self
                .emissive_texture
                .and_then(|i| textures.id_to_bindgroup_index(i))
                .unwrap_or_default() as u32,

            normal_texture: self
                .normal_texture
                .and_then(|i| textures.id_to_bindgroup_index(i))
                .unwrap_or_default() as u32,
            occlusion_texture: self
                .occlusion_texture
                .and_then(|i| textures.id_to_bindgroup_index(i))
                .unwrap_or_default() as u32,

            flags: self.calculate_flags(),
        }
    }

    pub fn calculate_flags(&self) -> MaterialFlags {
        let mut flags = MaterialFlags::empty();

        flags.set(
            MaterialFlags::HAS_BASE_COLOR_TEXTURE,
            self.base_color_texture.is_some(),
        );
        flags.set(
            MaterialFlags::HAS_METALLIC_ROUGHNESS_TEXTURE,
            self.metallic_roughness_texture.is_some(),
        );
        flags.set(
            MaterialFlags::HAS_EMISSIVE_TEXTURE,
            self.emissive_texture.is_some(),
        );
        flags.set(
            MaterialFlags::HAS_OCCLUSION_TEXTURE,
            self.occlusion_texture.is_some(),
        );
        flags.set(
            MaterialFlags::HAS_NORMAL_TEXTURE,
            self.normal_texture.is_some(),
        );

        flags
    }
}
