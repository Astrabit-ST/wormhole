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

    phantom: PhantomData<[T]>,
}

pub struct Writer<'buf, T> {
    internal: &'buf Buffer<T>,
    cpu_buffer: encase::UniformBuffer<Vec<u8>>, // FIXME: figure out how to do this on the stack with const generics?
}

impl<T> Buffer<T>
where
    T: encase::ShaderSize,
{
    pub fn new(
        render_state: &render::State,
        usage: wgpu::BufferUsages,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let gpu_buffer = render_state
            .wgpu
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("wormhole single buffer"),
                size: T::SHADER_SIZE.get() as wgpu::BufferAddress * 256,
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM
                    | usage,
                mapped_at_creation: false,
            });
        let bind_group = render_state
            .wgpu
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

            phantom: PhantomData,
        }
    }

    pub fn start_write(&self) -> Writer<'_, T> {
        Writer {
            internal: self,
            cpu_buffer: encase::UniformBuffer::new(Vec::with_capacity(
                self.gpu_buffer.size() as usize
            )),
        }
    }
}

impl<'buf, T> Writer<'buf, T>
where
    T: encase::ShaderType + encase::ShaderSize + encase::internal::WriteInto,
{
    pub fn write(&mut self, value: &T) -> Result<(), encase::internal::Error> {
        self.cpu_buffer.write(value)
    }

    pub fn finish(self, render_state: &render::State) -> &'buf wgpu::BindGroup {
        let cpu_buffer = self.cpu_buffer.into_inner();

        render_state
            .wgpu
            .queue
            .write_buffer(&self.internal.gpu_buffer, 0, &cpu_buffer);

        &self.internal.bind_group
    }
}
