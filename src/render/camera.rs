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
use winit::event::VirtualKeyCode;

#[derive(Debug)]
pub struct Camera {
    projection: Projection,
    transform: Transform,
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

        Camera {
            projection,
            transform,
        }
    }

    pub fn update(&mut self, input: &input::State, dt: f32) {
        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.transform.yaw.sin_cos();
        let forward = glam::Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = glam::Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if input.keyboard.held(VirtualKeyCode::W) {
            self.transform.position += forward * 4.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::A) {
            self.transform.position -= right * 4.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::S) {
            self.transform.position -= forward * 4.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::D) {
            self.transform.position += right * 4.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::Space) {
            self.transform.position.y += 4.0 * dt;
        }

        if input.keyboard.held(VirtualKeyCode::LShift) {
            self.transform.position.y -= 4.0 * dt;
        }

        let (mouse_x, mouse_y) = input.mouse.mouse_diff();
        let monitor_size = input.monitor_resolution();

        // Normalize based on monitor size.
        // This keeps mouse sensitivity consistent based on different resolutions.
        // (I hope)
        let norm_mouse_x = mouse_x / monitor_size.width as f32;
        let norm_mouse_y = mouse_y / monitor_size.height as f32;

        #[cfg(not(feature = "capture_mouse"))]
        if input.mouse.held(winit::event::MouseButton::Left) {
            self.transform.yaw += norm_mouse_x * 3.;
            self.transform.pitch -= norm_mouse_y * 3.;
        }
        #[cfg(feature = "capture_mouse")]
        {
            self.transform.yaw += norm_mouse_x * 5.;
            self.transform.pitch -= norm_mouse_y * 5.;
        }

        if let Some(size) = input.new_window_size() {
            self.projection.aspect = size.width as f32 / size.height as f32;
        }

        // Keep the camera's angle from going too high/low.
        if self.transform.pitch < -SAFE_FRAC_PI_2 {
            self.transform.pitch = -SAFE_FRAC_PI_2;
        } else if self.transform.pitch > SAFE_FRAC_PI_2 {
            self.transform.pitch = SAFE_FRAC_PI_2;
        }
    }
}

impl encase::ShaderSize for Camera {
    const SHADER_SIZE: std::num::NonZeroU64 = glam::Mat4::SHADER_SIZE;
}

impl encase::ShaderType for Camera {
    type ExtraMetadata = <glam::Mat4 as encase::ShaderType>::ExtraMetadata;
    const METADATA: encase::private::Metadata<Self::ExtraMetadata> =
        <glam::Mat4 as encase::ShaderType>::METADATA;
}

impl encase::internal::WriteInto for Camera {
    fn write_into<B>(&self, writer: &mut encase::internal::Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        (self.projection.build_projection_matrix() * self.transform.build_translation_matrix())
            .write_into(writer)
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
