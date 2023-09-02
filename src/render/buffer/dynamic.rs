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

pub struct Buffer<T> {
    gpu_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: &'static wgpu::BindGroupLayout,

    phantom: PhantomData<[T]>,
}

pub struct Writer<'buf, T> {
    cpu_buffer: encase::DynamicStorageBuffer<Vec<u8>>,
    internal: &'buf mut Buffer<T>,
}

impl<T> Buffer<T>
where
    T: encase::ShaderSize,
{
    pub fn new(
        render_state: &render::State,
        usage: wgpu::BufferUsages,
        bind_group_layout: &'static wgpu::BindGroupLayout,
    ) -> Self {
        let gpu_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wormhole dynamic buffer"),
            size: T::SHADER_SIZE.get() as wgpu::BufferAddress * 256,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE
                | usage,
            mapped_at_creation: false,
        });
        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("wormhole dynamic buffer bind group"),
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_buffer.as_entire_binding(),
                }],
            });

        Self {
            gpu_buffer,
            bind_group,
            bind_group_layout,

            phantom: PhantomData,
        }
    }

    pub fn start_write(&mut self) -> Writer<'_, T> {
        Writer {
            cpu_buffer: encase::DynamicStorageBuffer::new(Vec::with_capacity(
                self.gpu_buffer.size() as usize,
            )),
            internal: self,
        }
    }
}

impl<'buf, T> Writer<'buf, T>
where
    T: encase::ShaderType + encase::ShaderSize + encase::internal::WriteInto,
{
    pub fn push(&mut self, value: &T) -> Result<u64, encase::internal::Error> {
        self.cpu_buffer
            .write(value)
            .map(|offset| offset / T::SHADER_SIZE.get())
    }

    pub fn finish(self, render_state: &render::State) -> &'buf wgpu::BindGroup {
        let cpu_buffer = self.cpu_buffer.into_inner();
        if self.internal.gpu_buffer.size() < cpu_buffer.len() as wgpu::BufferAddress {
            let gpu_buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole dynamic buffer"),
                size: self.internal.gpu_buffer.size() * 2,
                usage: self.internal.gpu_buffer.usage(),
                mapped_at_creation: false,
            });

            let bind_group = render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("wormhole dynamic buffer bind group"),
                    layout: self.internal.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: gpu_buffer.as_entire_binding(),
                    }],
                });

            self.internal.gpu_buffer = gpu_buffer;
            self.internal.bind_group = bind_group;
        }
        render_state
            .queue
            .write_buffer(&self.internal.gpu_buffer, 0, &cpu_buffer);

        &self.internal.bind_group
    }
}
