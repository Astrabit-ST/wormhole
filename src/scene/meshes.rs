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

use crate::assets;
use crate::render;

#[derive(Debug)]
pub struct Meshes {
    vertex_buffers: VertexBuffers,
    index_buffer: Buffer<u32>,

    vertex_buffer_bind_group: wgpu::BindGroup,

    seen_meshes: HashMap<MeshRef, MeshIndex>,
}

#[derive(Debug)]
struct VertexBuffers {
    position: Buffer<glam::Vec3>,
    normal: Buffer<glam::Vec3>,
    tex_coord: Buffer<glam::Vec2>,
    color: Buffer<render::Color>,
    tangent: Buffer<glam::Vec4>,
}

#[derive(Debug)]
struct Buffer<T> {
    internal_buffer: wgpu::Buffer,

    written_len: wgpu::BufferAddress,

    unwritten: Vec<T>,
}

struct MeshRef(Arc<render::Mesh>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshIndex {
    pub position_offset: wgpu::BufferAddress,
    pub normal_offset: wgpu::BufferAddress,
    pub tex_coord_offset: wgpu::BufferAddress,
    pub color_offset: wgpu::BufferAddress,
    pub tangent_offset: wgpu::BufferAddress,

    pub index_offset: wgpu::BufferAddress,
    pub index_count: wgpu::BufferAddress,

    pub material_id: assets::MaterialId,
    pub mesh_flags: render::VertexFormat,
}

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

impl<T> Buffer<T>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    const SIZE: wgpu::BufferAddress = std::mem::size_of::<T>() as wgpu::BufferAddress;

    pub fn new(render_state: &render::State, usage: wgpu::BufferUsages) -> Self {
        let internal_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene meshes internal buffer"),
                size: Self::SIZE * 2_u64.pow(12),
                usage: usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        Self {
            internal_buffer,
            written_len: 0,
            unwritten: Vec::with_capacity(2048),
        }
    }

    pub fn queue_write(&mut self, values: &[T]) -> wgpu::BufferAddress {
        // Calculate offset based on current written len + the amount of unwritten values * the size of a value
        // Offset is in bytes
        let offset = self.written_len + (self.unwritten.len() as wgpu::BufferAddress * Self::SIZE);
        self.unwritten.extend(values);
        offset
    }

    // returns true if the buffer has been resized (this is used to recreate bind groups)
    pub fn write_unwritten(
        &mut self,
        render_state: &render::State,
        encoder: &mut wgpu::CommandEncoder,
    ) -> bool {
        // Calculate the final buffer size in bytes
        let final_buffer_size =
            self.written_len + (self.unwritten.len() as wgpu::BufferAddress * Self::SIZE);
        let needs_resize = final_buffer_size > self.internal_buffer.size();
        // If the buffer is too small, create a new one and copy the original data over
        if needs_resize {
            let new_buffer = render_state
                .wgpu
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("wormhole scene meshes internal buffer"),
                    size: final_buffer_size / 2 + final_buffer_size,
                    usage: self.internal_buffer.usage(),
                    mapped_at_creation: false,
                });
            encoder.copy_buffer_to_buffer(
                &self.internal_buffer,
                0,
                &new_buffer,
                0,
                self.written_len,
            );
            self.internal_buffer = new_buffer;
        }

        render_state.wgpu.queue.write_buffer(
            &self.internal_buffer,
            self.written_len,
            bytemuck::cast_slice(&self.unwritten),
        );

        self.unwritten.clear();
        self.written_len = final_buffer_size;

        needs_resize
    }
}

impl VertexBuffers {
    pub fn new(render_state: &render::State) -> Self {
        Self {
            position: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            normal: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            tex_coord: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            color: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            tangent: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
        }
    }

    pub fn create_bind_group(&self, render_state: &render::State) -> wgpu::BindGroup {
        render_state
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene meshes bind group"),
                layout: &render_state.bind_groups.vertex_data,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.position.internal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.normal.internal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.tex_coord.internal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.color.internal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.tangent.internal_buffer.as_entire_binding(),
                    },
                ],
            })
    }

    pub fn write_unwritten(
        &mut self,
        render_state: &render::State,
        encoder: &mut wgpu::CommandEncoder,
    ) -> bool {
        let mut resized = false;
        resized |= self.position.write_unwritten(render_state, encoder);
        resized |= self.normal.write_unwritten(render_state, encoder);
        resized |= self.tex_coord.write_unwritten(render_state, encoder);
        resized |= self.color.write_unwritten(render_state, encoder);
        resized |= self.tangent.write_unwritten(render_state, encoder);
        resized
    }
}

impl Meshes {
    pub fn new(render_state: &render::State) -> Self {
        let vertex_buffers = VertexBuffers::new(render_state);
        let vertex_buffer_bind_group = vertex_buffers.create_bind_group(render_state);

        Self {
            vertex_buffers,
            index_buffer: Buffer::new(render_state, wgpu::BufferUsages::INDEX),

            vertex_buffer_bind_group,

            seen_meshes: HashMap::with_capacity(16),
        }
    }

    pub fn upload_mesh(&mut self, mesh: Arc<render::Mesh>) -> MeshIndex {
        let mesh_ref = MeshRef(mesh.clone()); // FIXME: avoid extra clone
        if let Some(index) = self.seen_meshes.get(&mesh_ref).copied() {
            return index;
        }

        log::info!("Preparing to write mesh {mesh:#?}");

        let position_offset = self
            .vertex_buffers
            .position
            .queue_write(&mesh.parts.positions);

        let index_offset = self.index_buffer.queue_write(&mesh.indices);
        let index_count = mesh.indices.len() as wgpu::BufferAddress;

        let normal_offset = if let Some(n) = &mesh.parts.normals {
            self.vertex_buffers.normal.queue_write(n)
        } else {
            0
        };

        let tex_coord_offset = if let Some(t) = &mesh.parts.tex_coords {
            self.vertex_buffers.tex_coord.queue_write(t)
        } else {
            0
        };

        let color_offset = if let Some(c) = &mesh.parts.colors {
            self.vertex_buffers.color.queue_write(c)
        } else {
            0
        };

        let tangent_offset = if let Some(t) = &mesh.parts.tangents {
            self.vertex_buffers.tangent.queue_write(t)
        } else {
            0
        };

        MeshIndex {
            position_offset,
            normal_offset,
            tex_coord_offset,
            color_offset,
            tangent_offset,
            index_offset,
            index_count,
            material_id: mesh.material_id,
            mesh_flags: mesh.parts.vertex_format(),
        }
    }

    pub fn get_mesh_index(&self, mesh: Arc<render::Mesh>) -> Option<MeshIndex> {
        let mesh_ref = MeshRef(mesh);

        self.seen_meshes.get(&mesh_ref).copied()
    }

    pub fn write_unwritten(
        &mut self,
        render_state: &render::State,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.vertex_buffers.write_unwritten(render_state, encoder) {
            self.vertex_buffer_bind_group = self.vertex_buffers.create_bind_group(render_state);
        }

        self.index_buffer.write_unwritten(render_state, encoder);
    }

    pub fn as_bind_group_index_buffer(&self) -> (&wgpu::BindGroup, &wgpu::Buffer) {
        (
            &self.vertex_buffer_bind_group,
            &self.index_buffer.internal_buffer,
        )
    }
}

impl render::traits::Bindable for Meshes {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("scene meshes bind group layout"),
            entries: &[
                // positions
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true }, // FIXME: skinning might require write access
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // normals
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // tex coords
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // colors
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // tangents
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout {
        &render_state.bind_groups.vertex_data
    }
}
