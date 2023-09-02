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
use once_cell::sync::OnceCell;

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Quat,
}

impl Transform {
    fn to_matrix(self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position)
    }
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, 30_f32.to_radians()),
        }
    }
}

impl encase::ShaderSize for Transform {
    const SHADER_SIZE: std::num::NonZeroU64 = glam::Mat4::SHADER_SIZE;
}

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

impl Transform {
    pub fn create_bind_group_layout(render_state: &render::State) {
        let layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                });
        LAYOUT
            .set(layout)
            .expect("transform bind group layout already initialized");
    }

    pub fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        LAYOUT
            .get()
            .expect("transform bind group layout uninitialized")
    }
}
