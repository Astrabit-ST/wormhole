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
use std::marker::PhantomData;

// A dynamically growing GPU buffer.
// Useful for writing frame specific data that may change in amount
pub struct Buffer<T> {
    gpu_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    phantom: PhantomData<[T]>,
}

pub struct Writer<'buf, T> {
    cpu_buffer: encase::DynamicStorageBuffer<Vec<u8>>,
    internal: &'buf mut Buffer<T>,
}

impl<T> Buffer<T>
where
    T: render::traits::DynamicBufferWriteable,
{
    pub fn new(render_state: &render::State, usage: wgpu::BufferUsages) -> Self {
        let buffer_size = T::SHADER_SIZE.get() * 32;

        let gpu_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole dynamic buffer"),
                size: buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE
                    | usage,
                mapped_at_creation: false,
            });
        let bind_group = render_state
            .wgpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wormhole dynamic buffer bind group"),
                layout: T::get_layout(render_state),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_buffer.as_entire_binding(),
                }],
            });

        Self {
            gpu_buffer,
            bind_group,

            phantom: PhantomData,
        }
    }

    pub fn start_write(&mut self) -> Writer<'_, T> {
        Writer {
            cpu_buffer: encase::DynamicStorageBuffer::new_with_alignment(
                Vec::with_capacity(self.gpu_buffer.size() as usize),
                T::ALIGN,
            ),
            internal: self,
        }
    }
}

impl<'buf, T> Writer<'buf, T>
where
    T: render::traits::DynamicBufferWriteable,
{
    pub fn push(&mut self, value: &T) -> u64 {
        let offset = self
            .cpu_buffer
            .write(value)
            .expect("failed to write transform data");
        offset / wgpu::util::align_to(T::SHADER_SIZE.get(), T::ALIGN)
    }

    pub fn finish(self, render_state: &render::State) -> &'buf wgpu::BindGroup {
        let cpu_buffer = self.cpu_buffer.into_inner();
        if self.internal.gpu_buffer.size() < cpu_buffer.len() as wgpu::BufferAddress {
            let size = cpu_buffer.len();
            let size = (size / 2 + size) as wgpu::BufferAddress; // Multiply by 1.5

            let gpu_buffer = render_state
                .wgpu
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("wormhole dynamic buffer"),
                    size,
                    usage: self.internal.gpu_buffer.usage(),
                    mapped_at_creation: false,
                });

            let bind_group =
                render_state
                    .wgpu
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("wormhole dynamic buffer bind group"),
                        layout: T::get_layout(render_state),
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: gpu_buffer.as_entire_binding(),
                        }],
                    });

            self.internal.gpu_buffer = gpu_buffer;
            self.internal.bind_group = bind_group;
        }
        render_state
            .wgpu
            .queue
            .write_buffer(&self.internal.gpu_buffer, 0, &cpu_buffer);

        &self.internal.bind_group
    }
}
