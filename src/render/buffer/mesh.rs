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

pub struct Buffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

pub struct Writer<'buf> {
    cpu_vertex_buffer: Vec<u8>,
    cpu_index_buffer: Vec<u8>,

    internal: &'buf mut Buffer,
}

impl Buffer {
    pub fn new(render_state: &render::State, usage: wgpu::BufferUsages) -> Self {
        let vertex_buffer_size = std::mem::size_of::<render::Vertex>() * 1024;
        let index_buffer_size = std::mem::size_of::<u32>() * 2048;

        let vertex_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene vertex buffer"),
                size: vertex_buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::VERTEX
                    | usage,
                mapped_at_creation: false,
            });
        let index_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene index buffer"),
                size: index_buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::INDEX
                    | usage,
                mapped_at_creation: false,
            });

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn start_write(&mut self) -> Writer<'_> {
        Writer {
            cpu_vertex_buffer: Vec::with_capacity(self.vertex_buffer.size() as usize),
            cpu_index_buffer: Vec::with_capacity(self.index_buffer.size() as usize),

            internal: self,
        }
    }
}

impl<'buf> Writer<'buf> {
    pub fn push(&mut self, vertices: &[render::Vertex], indices: &[u32]) -> (u64, u64) {
        let vertex_offset = self.cpu_vertex_buffer.len() as u64;
        let index_offset = self.cpu_index_buffer.len() as u64;

        self.cpu_vertex_buffer
            .extend(bytemuck::cast_slice(vertices));
        self.cpu_index_buffer.extend(bytemuck::cast_slice(indices));

        (vertex_offset, index_offset)
    }

    pub fn finish(self, render_state: &render::State) -> (&'buf wgpu::Buffer, &'buf wgpu::Buffer) {
        if self.internal.vertex_buffer.size() < self.cpu_vertex_buffer.len() as wgpu::BufferAddress
        {
            let size = self.cpu_vertex_buffer.len();
            let size = (size / 2 + size) as wgpu::BufferAddress; // Multiply by 1.5

            let gpu_vertex_buffer =
                render_state
                    .wgpu
                    .device
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some("wormhole scene vertex buffer"),
                        size,
                        usage: self.internal.vertex_buffer.usage(),
                        mapped_at_creation: false,
                    });

            self.internal.vertex_buffer = gpu_vertex_buffer;
        }

        if self.internal.index_buffer.size() < self.cpu_index_buffer.len() as wgpu::BufferAddress {
            let size = self.cpu_index_buffer.len();
            let size = (size / 2 + size) as wgpu::BufferAddress; // Multiply by 1.5

            let gpu_index_buffer =
                render_state
                    .wgpu
                    .device
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some("wormhole scene index buffer"),
                        size,
                        usage: self.internal.index_buffer.usage(),
                        mapped_at_creation: false,
                    });

            self.internal.index_buffer = gpu_index_buffer;
        }

        render_state.wgpu.queue.write_buffer(
            &self.internal.vertex_buffer,
            0,
            &self.cpu_vertex_buffer,
        );

        render_state.wgpu.queue.write_buffer(
            &self.internal.index_buffer,
            0,
            &self.cpu_index_buffer,
        );

        (&self.internal.vertex_buffer, &self.internal.index_buffer)
    }
}
