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
use itertools::Itertools;

use crate::assets;
use crate::render;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub parts: MeshParts,
    pub indices: Vec<u32>,
    pub material_id: assets::MaterialId,
}

#[derive(Clone, Debug)]
pub struct MeshParts {
    pub positions: Vec<glam::Vec3>,

    pub normals: Option<Vec<glam::Vec3>>,
    pub tex_coords: Option<Vec<glam::Vec2>>,
    pub colors: Option<Vec<render::Color>>,
    pub tangents: Option<Vec<glam::Vec4>>,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct VertexFormat: u32 {
        const HAS_VTX_NORMALS   = 0b0000_0001;
        const HAS_TEX_COORDS    = 0b0000_0010;
        const HAS_VTX_COLOR     = 0b0000_0100;
        const HAS_VTX_TANGENT   = 0b0000_1000;
    }
}

impl MeshParts {
    pub fn vertex_format(&self) -> VertexFormat {
        let mut format = VertexFormat::empty();
        format.set(VertexFormat::HAS_TEX_COORDS, self.tex_coords.is_some());
        format.set(VertexFormat::HAS_VTX_COLOR, self.colors.is_some());
        format.set(VertexFormat::HAS_VTX_TANGENT, self.tangents.is_some());
        format
    }
}

impl Mesh {
    pub fn new(parts: &MeshParts, indices: &[u32], material_id: assets::MaterialId) -> Self {
        Self {
            parts: parts.clone(),
            indices: indices.to_vec(),
            material_id,
        }
    }
}

impl MeshParts {
    pub fn from_gltf_reader<'reader, F>(reader: gltf::mesh::Reader<'reader, 'reader, F>) -> Self
    where
        F: Clone + Fn(gltf::Buffer<'reader>) -> Option<&'reader [u8]>,
    {
        let positions = reader
            .read_positions()
            .expect("no positions provided")
            .map(glam::Vec3::from_array)
            .collect_vec();

        let normals = reader
            .read_normals()
            .map(|n| n.map(glam::Vec3::from_array).collect_vec());

        let tex_coords = reader
            .read_tex_coords(0)
            .map(|t| t.into_f32().map(glam::Vec2::from_array).collect_vec());

        let colors = reader
            .read_colors(0)
            .map(|c| c.into_rgba_f32().map(render::Color::from).collect_vec());

        let tangents = reader
            .read_tangents()
            .map(|t| t.map(glam::Vec4::from_array).collect_vec());

        Self {
            positions,
            normals,
            tex_coords,
            colors,
            tangents,
        }
    }

    pub fn approximate_tangents(&mut self, indices: &[u32]) {
        self.tangents.get_or_insert_with(|| todo!());
    }
}

impl Mesh {
    /// The mesh must be loaded with `triangluate` and `single_index` set to true.
    pub fn from_tobj_mesh(mut mesh: tobj::Mesh) -> Self {
        mesh.positions.shrink_to_fit();
        let positions = bytemuck::cast_vec(mesh.positions);

        mesh.normals.shrink_to_fit();
        let normals = if mesh.normals.is_empty() {
            None
        } else {
            Some(bytemuck::cast_vec(mesh.normals))
        };

        mesh.texcoords.shrink_to_fit();
        let tex_coords = if mesh.texcoords.is_empty() {
            None
        } else {
            Some(bytemuck::cast_vec(mesh.texcoords))
        };

        mesh.vertex_color.shrink_to_fit();
        let colors = if mesh.vertex_color.is_empty() {
            None
        } else {
            Some(bytemuck::cast_vec(mesh.vertex_color))
        };

        let mut parts = MeshParts {
            positions,
            normals,
            tex_coords,
            colors,
            tangents: None,
        };
        parts.approximate_tangents(&mesh.indices);

        Self {
            parts,
            indices: mesh.indices,
            material_id: assets::MaterialId::Path(0), // FIXME
        }
    }

    pub fn from_gltf_primitive(
        gltf_id: assets::GltfId,
        primitive: gltf::Primitive<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let indices = reader.read_indices().unwrap().into_u32().collect_vec();

        let mut parts = MeshParts::from_gltf_reader(reader);
        parts.approximate_tangents(&indices);

        Self {
            parts,
            indices,
            material_id: assets::MaterialId::Gltf(
                gltf_id,
                primitive.material().index().unwrap_or_default(),
            ),
        }
    }
}

impl From<tobj::Mesh> for Mesh {
    fn from(value: tobj::Mesh) -> Self {
        Self::from_tobj_mesh(value)
    }
}
