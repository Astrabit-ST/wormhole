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

pub struct Camera {
    data: Data,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
struct Data {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array_2d(&[
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.5],
    [0.0, 0.0, 0.0, 1.0],
]);

impl Data {
    fn build_view_projection_matrix(self) -> glam::Mat4 {
        let view = ::glam::Mat4::look_at_rh(self.eye, self.target, self.up);

        let proj = glam::Mat4::perspective_rh_gl(
            self.fovy.to_radians(),
            self.aspect,
            self.znear,
            self.zfar,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Camera {
    pub fn new(render_state: &render::State) -> Self {
        let data = Data {
            eye: glam::vec3(2.0, 2.0, 4.0),
            // have it look at the origin
            target: glam::Vec3::ZERO,
            // which way is "up"
            up: glam::Vec3::Y,
            aspect: render_state.surface_config.width as f32
                / render_state.surface_config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let proj = data.build_view_projection_matrix();

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[proj]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: Camera::bind_group_layout(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Camera {
            data,
            buffer,
            bind_group,
        }
    }

    pub fn reupload(&self, state: &render::State) {
        let proj = self.data.build_view_projection_matrix();

        state
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[proj]));
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>, index: u32) {
        render_pass.set_bind_group(index, &self.bind_group, &[]);
    }
}

impl Camera {
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
