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

pub struct BindGroupLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(
        mut self,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BindingType,
        count: Option<std::num::NonZeroU32>,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty,
            count,
        });
        self
    }

    #[must_use]
    pub fn build(self, device: &wgpu::Device, label: wgpu::Label<'_>) -> wgpu::BindGroupLayout {
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &self.entries,
        };
        device.create_bind_group_layout(&descriptor)
    }
}

pub struct BindGroupBuilder<'res> {
    entries: Vec<wgpu::BindGroupEntry<'res>>,
}

impl<'res> BindGroupBuilder<'res> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(mut self, resource: wgpu::BindingResource<'res>) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource,
        });
        self
    }

    pub fn append_buffer(self, buffer: &'res wgpu::Buffer) -> Self {
        self.append(buffer.as_entire_binding())
    }

    pub fn append_buffer_with_size(self, buffer: &'res wgpu::Buffer, size: u64) -> Self {
        self.append(wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(size),
        }))
    }

    pub fn append_sampler(self, sampler: &'res wgpu::Sampler) -> Self {
        self.append(wgpu::BindingResource::Sampler(sampler))
    }

    pub fn append_sampler_array(self, sampler_array: &'res [&'res wgpu::Sampler]) -> Self {
        self.append(wgpu::BindingResource::SamplerArray(sampler_array))
    }

    pub fn append_texture_view(self, texture: &'res wgpu::TextureView) -> Self {
        self.append(wgpu::BindingResource::TextureView(texture))
    }

    pub fn append_texture_view_array(
        self,
        texture_view_array: &'res [&'res wgpu::TextureView],
    ) -> Self {
        self.append(wgpu::BindingResource::TextureViewArray(texture_view_array))
    }

    #[must_use]
    pub fn build(
        self,
        device: &wgpu::Device,
        label: wgpu::Label<'_>,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let descriptor = wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &self.entries,
        };
        device.create_bind_group(&descriptor)
    }
}
