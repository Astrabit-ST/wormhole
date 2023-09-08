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
use crate::scene;
use itertools::Itertools;

pub struct Mesh {
    pub vertices: Vec<render::Vertex>,
    pub indices: Vec<u32>,
}

pub struct PreparedMesh {
    vertex_offset: u64,
    index_offset: u64,

    vertex_count: u64,
    index_count: u64,
}

impl Mesh {
    pub fn new(vertices: &[render::Vertex], indices: &[u32]) -> Self {
        Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        }
    }

    /// The mesh must be loaded with `triangluate` and `single_index` set to true.
    pub fn from_tobj_mesh(mesh: &tobj::Mesh) -> Self {
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
        let mut triangles_included = vec![0; vertices.len()];

        for i in mesh.indices.chunks(3) {
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

        Self {
            vertices,
            indices: mesh.indices.clone(),
        }
    }

    pub fn prepare(&self, resources: &mut scene::PrepareResources<'_>) -> PreparedMesh {
        let (vertex_offset, index_offset) = resources.mesh.push(&self.vertices, &self.indices);

        let vertex_count = self.vertices.len() as u64;
        let index_count = self.indices.len() as u64;

        PreparedMesh {
            vertex_offset,
            index_offset,
            vertex_count,
            index_count,
        }
    }
}

impl PreparedMesh {
    pub fn draw<'rpass>(
        self,
        resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        let vertex_start = self.vertex_offset;
        let vertex_end =
            self.vertex_offset + (self.vertex_count * std::mem::size_of::<render::Vertex>() as u64);

        let index_start = self.index_offset;
        let index_end = self.index_offset + (self.index_count * std::mem::size_of::<u32>() as u64);

        render_pass.set_vertex_buffer(0, resources.vertices.slice(vertex_start..vertex_end));
        render_pass.set_index_buffer(
            resources.indices.slice(index_start..index_end),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..(self.index_count as u32), 0, 0..1)
    }
}
