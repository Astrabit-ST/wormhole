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
    pub scale: glam::Vec3,
}

#[derive(encase::ShaderType, Debug)]
pub struct Data {
    obj_proj: glam::Mat4,
    normal_proj: glam::Mat3,
}

impl Transform {
    fn to_obj_proj(self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    fn to_normal_proj(self) -> glam::Mat3 {
        glam::Mat3::from_quat(self.rotation)
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_position(glam::vec3(x, y, z))
    }

    pub fn from_position(position: glam::Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn from_position_rotation(position: glam::Vec3, rotation: glam::Quat) -> Self {
        Self {
            position,
            rotation,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn translate(&mut self, vec: glam::Vec3) {
        self.position += vec
    }
}

impl encase::ShaderSize for Transform {}

impl encase::ShaderType for Transform {
    type ExtraMetadata = <Data as encase::ShaderType>::ExtraMetadata;
    const METADATA: encase::private::Metadata<Self::ExtraMetadata> =
        <Data as encase::ShaderType>::METADATA;
}

impl encase::internal::WriteInto for Transform {
    fn write_into<B>(&self, writer: &mut encase::internal::Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        let obj_proj = self.to_obj_proj();
        let normal_proj = self.to_normal_proj();
        let data = Data {
            obj_proj,
            normal_proj,
        };
        data.write_into(writer)
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
