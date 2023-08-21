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
use wgpu::util::DeviceExt;

pub struct Transform {
    data: Data,

    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
struct Data {
    position: glam::Vec3,
    rotation: glam::Quat,
}

impl Data {
    fn to_matrix(self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position)
    }
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Transform {
    pub fn new(render_state: &render::State) -> Self {
        let data = Data {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, 30_f32.to_radians()),
        };

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("transform buffer"),
                contents: bytemuck::bytes_of(&data.to_matrix()),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("transform bind group layout"),
                layout: Transform::bind_group_layout(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            data,
            bind_group,
            buffer,
        }
    }

    pub fn reupload(&self, render_state: &render::State) {
        // WGSL doesn't have a decent concept of a quaternion.
        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer
        render_state.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::bytes_of(&self.data.to_matrix()),
        );
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>, index: u32) {
        render_pass.set_bind_group(index, &self.bind_group, &[]);
    }
}

impl Transform {
    pub fn create_bind_group_layout(render_state: &render::State) {
        let layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        LAYOUT
            .set(layout)
            .expect("camera bind group layout already initialized");
    }

    pub fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        LAYOUT.get().expect("layout uninitialized")
    }
}
