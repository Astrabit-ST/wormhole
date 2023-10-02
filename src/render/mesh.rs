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
use itertools::Itertools;

pub struct Mesh {
    pub vertices: Vec<render::Vertex>,
    pub indices: Vec<u32>,
    pub material_id: usize,
}

impl Mesh {
    pub fn new(vertices: &[render::Vertex], indices: &[u32], material_id: usize) -> Self {
        Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            material_id,
        }
    }

    /// The mesh must be loaded with `triangluate` and `single_index` set to true.
    pub fn from_tobj_mesh(mesh: tobj::Mesh) -> Self {
        // Create a list of vertices from the mesh.
        let mut tex_coords = bytemuck::cast_slice(&mesh.texcoords).iter().copied();
        let mut normals = bytemuck::cast_slice(&mesh.normals).iter().copied();

        let mut vertices = bytemuck::cast_slice(&mesh.positions)
            .iter()
            .copied()
            .map(|position| {
                let tex_coords = tex_coords.next().unwrap_or_default();
                let normal = normals.next().unwrap_or_default();

                render::Vertex {
                    position,
                    tex_coords,
                    normal,

                    ..Default::default()
                }
            })
            .collect_vec();

        Self::calculate_bitangent_tangent(&mesh.indices, &mut vertices);

        Self {
            vertices,
            indices: mesh.indices,
            material_id: mesh.material_id.unwrap_or_default(),
        }
    }

    pub fn from_gltf_primitive(
        primitive: gltf::Primitive<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader.read_positions().unwrap().map(glam::Vec3::from_array);

        let mut normals = reader.read_normals().unwrap().map(glam::Vec3::from_array);

        let mut tex_coords = reader
            .read_tex_coords(0)
            .unwrap()
            .into_f32()
            .map(glam::Vec2::from_array);

        let mut vertices = positions
            .map(|position| {
                let tex_coords = tex_coords.next().unwrap_or_default();
                let normal = normals.next().unwrap_or_default();

                render::Vertex {
                    position,
                    tex_coords,
                    normal,

                    ..Default::default()
                }
            })
            .collect_vec();

        let indices = reader.read_indices().unwrap().into_u32().collect_vec();

        Self::calculate_bitangent_tangent(&indices, &mut vertices);

        Self {
            vertices,
            indices,
            material_id: primitive.material().index().unwrap_or_default(),
        }
    }

    fn calculate_bitangent_tangent(indices: &[u32], vertices: &mut [render::Vertex]) {
        let mut triangles_included = vec![0; vertices.len()];

        for i in indices.chunks(3) {
            let v0 = vertices[i[0] as usize];
            let v1 = vertices[i[1] as usize];
            let v2 = vertices[i[2] as usize];

            let delta_pos1 = v1.position - v0.position;
            let delta_pos2 = v2.position - v0.position;

            let delta_uv1 = v1.tex_coords - v0.tex_coords;
            let delta_uv2 = v2.tex_coords - v0.tex_coords;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            // We flip the bitangent to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            // We'll use the same tangent/bitangent for each vertex in the triangle
            vertices[i[0] as usize].tangent = tangent + v0.tangent;
            vertices[i[1] as usize].tangent = tangent + v1.tangent;
            vertices[i[2] as usize].tangent = tangent + v2.tangent;

            vertices[i[0] as usize].bitangent = bitangent + v0.bitangent;
            vertices[i[1] as usize].bitangent = bitangent + v1.bitangent;
            vertices[i[2] as usize].bitangent = bitangent + v2.bitangent;

            triangles_included[i[0] as usize] += 1;
            triangles_included[i[1] as usize] += 1;
            triangles_included[i[2] as usize] += 1;
        }
        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let v = &mut vertices[i];
            v.tangent *= denom;
            v.bitangent *= denom;
        }
    }
}

impl From<tobj::Mesh> for Mesh {
    fn from(value: tobj::Mesh) -> Self {
        Self::from_tobj_mesh(value)
    }
}

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("vertices", &self.vertices.len())
            .field("indices", &self.indices.len())
            .finish()
    }
}
