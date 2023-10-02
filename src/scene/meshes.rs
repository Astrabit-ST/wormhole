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
use std::collections::HashMap;
use std::sync::Arc;

use crate::render;
use crate::scene;

#[derive(Debug)]
pub struct Meshes {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    written_vertex_len: wgpu::BufferAddress,
    written_index_len: wgpu::BufferAddress,

    unwritten_vertex_offset: wgpu::BufferAddress,
    unwritten_index_offset: wgpu::BufferAddress,

    written_meshes: HashMap<MeshRef, MeshIndex>,
    unwritten_meshes: HashMap<MeshRef, MeshIndex>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshIndex {
    pub vertex_offset: wgpu::BufferAddress,
    pub vertex_count: wgpu::BufferAddress,

    pub index_offset: wgpu::BufferAddress,
    pub index_count: wgpu::BufferAddress,
}

struct MeshRef(Arc<render::Mesh>);

impl std::fmt::Debug for MeshRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::hash::Hash for MeshRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0);
        ptr.hash(state)
    }
}

impl PartialEq for MeshRef {
    fn eq(&self, other: &Self) -> bool {
        let self_ptr = Arc::as_ptr(&self.0);
        let other_ptr = Arc::as_ptr(&other.0);
        self_ptr == other_ptr
    }
}

impl Eq for MeshRef {}

const VERTEX_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<render::Vertex>() as wgpu::BufferAddress;
const INDEX_SIZE: wgpu::BufferAddress = std::mem::size_of::<u32>() as wgpu::BufferAddress;

impl Meshes {
    pub fn new(render_state: &render::State) -> Self {
        let vertex_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene vertices"),
                size: VERTEX_SIZE * 2_u64.pow(13),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        let index_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene vertices"),
                size: INDEX_SIZE * 2_u64.pow(14),
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        Self {
            vertex_buffer,
            index_buffer,

            unwritten_vertex_offset: 0,
            unwritten_index_offset: 0,

            written_vertex_len: 0,
            written_index_len: 0,

            written_meshes: HashMap::with_capacity(16),
            unwritten_meshes: HashMap::with_capacity(16),
        }
    }

    pub fn upload_mesh(&mut self, mesh: Arc<render::Mesh>) -> MeshIndex {
        if let Some(index) = self.get_mesh_index(Arc::clone(&mesh)) {
            return index;
        }

        let mesh_index = MeshIndex {
            vertex_offset: self.unwritten_vertex_offset,
            vertex_count: mesh.vertices.len() as wgpu::BufferAddress,

            index_offset: self.unwritten_index_offset,
            index_count: mesh.indices.len() as wgpu::BufferAddress,
        };

        self.unwritten_vertex_offset += mesh_index.vertex_count * VERTEX_SIZE;
        self.unwritten_index_offset += mesh_index.index_count * INDEX_SIZE;

        self.unwritten_meshes.insert(MeshRef(mesh), mesh_index);

        mesh_index
    }

    pub fn get_mesh_index(&self, mesh: Arc<render::Mesh>) -> Option<MeshIndex> {
        let mesh_ref = MeshRef(mesh);

        self.written_meshes
            .get(&mesh_ref)
            .or(self.unwritten_meshes.get(&mesh_ref))
            .copied()
    }

    pub fn write_unwritten(
        &mut self,
        render_state: &render::State,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.unwritten_vertex_offset > self.vertex_buffer.size() {
            log::info!(
                "Copying vertices to new buffer: {:#?}",
                0..(self.written_index_len / VERTEX_SIZE)
            );

            let new_buffer = render_state
                .wgpu
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("wormhole scene vertices"),
                    size: self.unwritten_vertex_offset + self.unwritten_vertex_offset / 2,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_SRC
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            encoder.copy_buffer_to_buffer(
                &self.vertex_buffer,
                0,
                &new_buffer,
                0,
                self.written_vertex_len,
            );
            self.vertex_buffer = new_buffer;
        }

        if self.unwritten_index_offset > self.index_buffer.size() {
            log::info!(
                "Copying indices to new buffer: {:#?}",
                0..(self.written_index_len / INDEX_SIZE)
            );
            let new_buffer = render_state
                .wgpu
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("wormhole scene indices"),
                    size: self.unwritten_index_offset + self.unwritten_index_offset / 2,
                    usage: wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_SRC
                        | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            encoder.copy_buffer_to_buffer(
                &self.index_buffer,
                0,
                &new_buffer,
                0,
                self.written_index_len,
            );
            self.index_buffer = new_buffer;
        }

        for (mesh, index) in self.unwritten_meshes.drain() {
            log::info!("Writing mesh {:#?}", mesh);

            render_state.wgpu.queue.write_buffer(
                &self.vertex_buffer,
                index.vertex_offset,
                bytemuck::cast_slice(&mesh.0.vertices),
            );
            render_state.wgpu.queue.write_buffer(
                &self.index_buffer,
                index.index_offset,
                bytemuck::cast_slice(&mesh.0.indices),
            );

            assert_eq!(
                None,
                self.written_meshes.insert(mesh, index),
                "There should be no written mesh duplicates"
            );
        }
        self.written_vertex_len = self.unwritten_vertex_offset;
        self.written_index_len = self.unwritten_index_offset;
    }

    pub fn as_buffers(&self) -> (&wgpu::Buffer, &wgpu::Buffer) {
        (&self.vertex_buffer, &self.index_buffer)
    }
}

impl MeshIndex {
    pub fn draw<'rpass>(
        self,
        resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        let vertex_start = self.vertex_offset;
        let vertex_end = self.vertex_offset + (self.vertex_count * VERTEX_SIZE);

        let index_start = self.index_offset;
        let index_end = self.index_offset + (self.index_count * INDEX_SIZE);

        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(vertex_start..vertex_end));
        render_pass.set_index_buffer(
            resources.index_buffer.slice(index_start..index_end),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..(self.index_count as u32), 0, 0..1)
    }
}
