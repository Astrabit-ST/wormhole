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

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Transform {
    fn to_matrix(self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position)
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
        }
    }

    pub fn from_position(position: glam::Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::IDENTITY,
        }
    }

    pub fn from_position_rotation(position: glam::Vec3, rotation: glam::Quat) -> Self {
        Self { position, rotation }
    }
}

impl Transform {
    pub fn translate(&mut self, vec: glam::Vec3) {
        self.position += vec
    }
}

impl encase::ShaderSize for Transform {}

impl encase::ShaderType for Transform {
    type ExtraMetadata = <glam::Mat4 as encase::ShaderType>::ExtraMetadata;
    const METADATA: encase::private::Metadata<Self::ExtraMetadata> =
        <glam::Mat4 as encase::ShaderType>::METADATA;
}

impl encase::internal::WriteInto for Transform {
    fn write_into<B>(&self, writer: &mut encase::internal::Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        self.to_matrix().write_into(writer)
    }
}

impl render::traits::Bindable for Transform {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("transform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
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

impl render::traits::DynamicBufferWriteable for Transform {}
