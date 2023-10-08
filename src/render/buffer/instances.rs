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
    instance_buffer: wgpu::Buffer,
}

pub struct Writer<'buf> {
    cpu_instance_buffer: Vec<u8>,

    internal: &'buf mut Buffer,
}

impl Buffer {
    pub fn new(render_state: &render::State, usage: wgpu::BufferUsages) -> Self {
        let instance_buffer_size = std::mem::size_of::<render::Instance>() * 1024;

        let instance_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole scene vertex buffer"),
                size: instance_buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::VERTEX
                    | usage,
                mapped_at_creation: false,
            });

        Self { instance_buffer }
    }

    pub fn start_write(&mut self) -> Writer<'_> {
        Writer {
            cpu_instance_buffer: Vec::with_capacity(self.instance_buffer.size() as usize),

            internal: self,
        }
    }
}

impl<'buf> Writer<'buf> {
    pub fn push(&mut self, instance: render::Instance) -> u64 {
        let instance_offset =
            self.cpu_instance_buffer.len() as u64 / std::mem::size_of::<render::Instance>() as u64;

        self.cpu_instance_buffer
            .extend(bytemuck::bytes_of(&instance));

        instance_offset
    }

    pub fn finish(self, render_state: &render::State) -> &'buf wgpu::Buffer {
        if self.internal.instance_buffer.size()
            < self.cpu_instance_buffer.len() as wgpu::BufferAddress
        {
            let size = self.cpu_instance_buffer.len();
            let size = (size / 2 + size) as wgpu::BufferAddress; // Multiply by 1.5

            let new_instance_buffer =
                render_state
                    .wgpu
                    .device
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some("wormhole scene instance buffer"),
                        size,
                        usage: self.internal.instance_buffer.usage(),
                        mapped_at_creation: false,
                    });

            self.internal.instance_buffer = new_instance_buffer;
        }

        render_state.wgpu.queue.write_buffer(
            &self.internal.instance_buffer,
            0,
            &self.cpu_instance_buffer,
        );

        &self.internal.instance_buffer
    }
}
