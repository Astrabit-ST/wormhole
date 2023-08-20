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
use crate::{input, render};

use once_cell::sync::OnceCell;
use wgpu::util::DeviceExt;
use winit::event::VirtualKeyCode;

#[derive(Debug)]
pub struct Camera {
    projection: Projection,
    transform: Transform,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
struct Transform {
    position: glam::Vec3,

    yaw: f32,
    pitch: f32,
}

impl Transform {
    fn build_translation_matrix(&self) -> glam::Mat4 {
        // glam::Mat4::look_at_rh(glam::vec3(0.0, 1.0, 2.0), glam::Vec3::ZERO, glam::Vec3::Y)
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        glam::Mat4::look_to_rh(
            self.position,
            glam::Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            glam::Vec3::Y,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
struct Projection {
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

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

impl Projection {
    fn build_projection_matrix(self) -> glam::Mat4 {
        OPENGL_TO_WGPU_MATRIX
            * glam::Mat4::perspective_rh_gl(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Camera {
    pub fn new(render_state: &render::State) -> Self {
        let transform = Transform {
            position: glam::Vec3::new(0.0, 0.0, 0.0),

            yaw: -90_f32.to_radians(),
            pitch: -20_f32.to_radians(),
        };
        let projection = Projection {
            aspect: render_state.surface_config.width as f32
                / render_state.surface_config.height as f32,
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let view_matrix =
            transform.build_translation_matrix() * projection.build_projection_matrix();

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[view_matrix]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            projection,
            transform,

            buffer,
            bind_group,
        }
    }

    pub fn update(&mut self, render_state: &render::State, input: &input::State) {
        let dt = 1.0 / 144.0;

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.transform.yaw.sin_cos();
        let forward = glam::Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = glam::Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if input.keyboard.held(VirtualKeyCode::W) {
            self.transform.position += forward * 2.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::A) {
            self.transform.position -= right * 2.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::S) {
            self.transform.position -= forward * 2.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::D) {
            self.transform.position += right * 2.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::Space) {
            self.transform.position.y += 2.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::LShift) {
            self.transform.position.y -= 2.0 * dt;
        }

        let (mx, my) = input.mouse.mouse_diff();
        self.transform.yaw += mx * 0.4 * dt;
        self.transform.pitch -= my * 0.4 * dt;

        // Keep the camera's angle from going too high/low.
        if self.transform.pitch < -SAFE_FRAC_PI_2 {
            self.transform.pitch = -SAFE_FRAC_PI_2;
        } else if self.transform.pitch > SAFE_FRAC_PI_2 {
            self.transform.pitch = SAFE_FRAC_PI_2;
        }

        self.reupload(render_state);
    }

    pub fn reupload(&self, render_state: &render::State) {
        let view_matrix =
            self.projection.build_projection_matrix() * self.transform.build_translation_matrix();

        render_state
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[view_matrix]));
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
