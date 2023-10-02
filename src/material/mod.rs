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

mod textures;
pub use textures::Textures;

pub struct Material {
    pub base_color: render::Color,
    pub base_color_texture: Option<assets::TextureId>,

    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<assets::TextureId>,

    pub emissive: render::Color,
    pub emissive_texture: Option<assets::TextureId>,

    pub occlusion_texture: Option<assets::TextureId>,

    pub flags: MaterialFlags,
}

#[derive(encase::ShaderType, Debug)]
pub struct Data {
    color: render::Color,
    metallic: f32,
    roughness: f32,
    emissive: f32,
    flags: u32,
}

bitflags::bitflags! {
    pub struct MaterialFlags: u32 {
        const HAS_BASE_COLOR_TEXTURE = 0b0000_0001;
        const HAS_METALLIC_ROUGHNESS_TEXTURE = 0b0000_0010;
        const HAS_EMISSIVE_TEXTURE = 0b0000_0100;
        const HAS_OCCLUSION_TEXTURE = 0b0000_1000;
    }
}

impl Material {
    pub fn from_gltf(material: gltf::Material<'_>) {}
}
