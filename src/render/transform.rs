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
    position: glam::Vec3,
    rotation: glam::Quat,

    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Transform {
    pub fn new(render_state: &render::State) -> Self {
        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("transform buffer"),
                contents: bytemuck::bytes_of(&glam::Mat4::IDENTITY),
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
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            bind_group,
            buffer,
        }
    }

    pub fn reupload(&self, render_state: &render::State) {
        // WGSL doesn't have a decent concept of a quaternion.
        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer
        let matrix =
            glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation);
        render_state
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[matrix]));
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
