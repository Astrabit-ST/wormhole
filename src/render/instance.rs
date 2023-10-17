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
use crate::scene;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position_offset: u32,
    pub normal_offset: u32,
    pub tex_coord_offset: u32,
    pub color_offset: u32,
    pub tangent_offset: u32,

    pub mesh_flags: render::VertexFormat,

    pub transform_index: u32,
    pub material_index: u32,
}

impl Instance {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32, 2 => Uint32, 3 => Uint32, 4 => Uint32, 5 => Uint32, 6 => Uint32, 7 => Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRS,
        }
    }

    pub fn from_mesh_transform_indices_without_material(
        mesh_index: scene::MeshIndex,
        transform_index: u32,
    ) -> Self {
        Self {
            position_offset: mesh_index.position_offset as u32,
            normal_offset: mesh_index.normal_offset as u32,
            tex_coord_offset: mesh_index.tex_coord_offset as u32,
            color_offset: mesh_index.color_offset as u32,
            tangent_offset: mesh_index.tangent_offset as u32,

            mesh_flags: mesh_index.mesh_flags,

            transform_index,
            material_index: 0,
        }
    }

    pub fn from_mesh_transform_indices_with_materials(
        mesh_index: scene::MeshIndex,
        transform_index: u32,
        materials: &assets::Materials,
    ) -> Self {
        Self {
            position_offset: mesh_index.position_offset as u32,
            normal_offset: mesh_index.normal_offset as u32,
            tex_coord_offset: mesh_index.tex_coord_offset as u32,
            color_offset: mesh_index.color_offset as u32,
            tangent_offset: mesh_index.tangent_offset as u32,

            mesh_flags: mesh_index.mesh_flags,

            transform_index,
            material_index: materials
                .id_to_bindgroup_index(mesh_index.material_id)
                .unwrap_or_default() as u32,
        }
    }
}
