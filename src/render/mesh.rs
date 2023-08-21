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
use wgpu::util::DeviceExt;

pub struct Mesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn new(render_state: &render::State, vertices: &[render::Vertex], indices: &[u32]) -> Self {
        let index_count = indices.len() as u32;
        let vertices = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh vertices"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("mesh vertices"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertices,
            indices,
            index_count,
        }
    }

    /// The mesh must be loaded with `triangluate` and `single_index` set to true.
    pub fn from_tobj_mesh(render_state: &render::State, mesh: &tobj::Mesh) -> Self {
        // Create a list of vertices from the mesh.
        let mut tex_coords = bytemuck::cast_slice(&mesh.texcoords).iter().copied();
        let mut normals = bytemuck::cast_slice(&mesh.normals).iter().copied();

        let vertices = bytemuck::cast_slice(&mesh.positions)
            .iter()
            .copied()
            .map(|position| {
                let tex_coords = tex_coords.next().unwrap_or_default();
                let normal = normals.next().unwrap_or_default();

                render::Vertex {
                    position,
                    tex_coords,
                    normal,
                }
            })
            .collect_vec();

        Self::new(render_state, &vertices, &mesh.indices)
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
