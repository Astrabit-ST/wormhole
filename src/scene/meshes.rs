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
use crate::scene;

#[derive(Debug)]
pub struct Meshes {
    position_buffer: Buffer<glam::Vec3>,
    normal_buffer: Buffer<glam::Vec3>,
    tex_coord_buffer: Buffer<glam::Vec2>,
    color_buffer: Buffer<render::Color>,
    tangent_buffer: Buffer<glam::Vec4>,
    index_buffer: Buffer<u32>,

    seen_meshes: HashMap<MeshRef, MeshIndex>,
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
    pub vertex_offset: wgpu::BufferAddress,
    pub vertex_count: wgpu::BufferAddress,

    pub index_offset: wgpu::BufferAddress,
    pub index_count: wgpu::BufferAddress,

    pub material_id: assets::MaterialId,
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

impl Meshes {
    pub fn new(render_state: &render::State) -> Self {
        Self {
            position_buffer: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            normal_buffer: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            tex_coord_buffer: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            color_buffer: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            tangent_buffer: Buffer::new(render_state, wgpu::BufferUsages::STORAGE),
            index_buffer: Buffer::new(render_state, wgpu::BufferUsages::INDEX),

            seen_meshes: HashMap::with_capacity(16),
        }
    }

    pub fn upload_mesh(&mut self, mesh: Arc<render::Mesh>) -> MeshIndex {
        let mesh_ref = MeshRef(mesh);
        if let Some(index) = self.seen_meshes.get(&mesh_ref).copied() {
            return index;
        }

        log::info!("Preparing to write mesh {mesh:#?}");

        mesh_index
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
        let mut recreate_bind_group = false;
        recreate_bind_group |= self.position_buffer.write_unwritten(render_state, encoder);
        recreate_bind_group |= self.normal_buffer.write_unwritten(render_state, encoder);
        recreate_bind_group |= self.tex_coord_buffer.write_unwritten(render_state, encoder);
        recreate_bind_group |= self.color_buffer.write_unwritten(render_state, encoder);
        recreate_bind_group |= self.tangent_buffer.write_unwritten(render_state, encoder);

        self.index_buffer.write_unwritten(render_state, encoder);
    }
}

impl MeshIndex {
    pub fn draw<'rpass>(
        self,
        render_resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        let base_vertex = (self.vertex_offset / VERTEX_SIZE) as i32;

        if let Some(material) = render_resources.assets.materials.get(self.material_id) {
            render_pass.set_bind_group(2, &material.bind_group, &[]);
        }

        render_pass.draw_indexed(0..(self.index_count as u32), base_vertex, 0..1)
    }
}

impl render::traits::Bindable for Meshes {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("scene meshes bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout {
        &render_state.bind_groups.transform
    }
}
