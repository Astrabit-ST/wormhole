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

use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Camera {
    pub projection: Projection,
    pub transform: Transform,
}

pub struct Data {
    pub view_pos: glam::Vec3,
    pub view_proj: glam::Mat4,
}

#[derive(Clone, Copy, Debug, Default)]
#[derive(PartialEq)]
pub struct Transform {
    position: glam::Vec3,

    yaw: f32,
    pitch: f32,
}

impl Transform {
    fn build_translation_matrix(&self) -> glam::Mat4 {
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
pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

impl Projection {
    fn build_projection_matrix(self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

impl Camera {
    pub fn new(render_state: &render::State) -> Self {
        let transform = Transform {
            position: glam::Vec3::new(0.0, 9.0, 0.0),

            yaw: 0.0,
            pitch: -90_f32.to_radians(),
        };
        let surface_config = render_state.wgpu.surface_config.lock();
        let projection = Projection {
            aspect: surface_config.width as f32 / surface_config.height as f32,
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

        if input.keyboard.held(KeyCode::KeyW) {
            self.transform.position += forward * 4.0 * dt;
        }

        if input.keyboard.held(KeyCode::KeyA) {
            self.transform.position -= right * 4.0 * dt;
        }

        if input.keyboard.held(KeyCode::KeyS) {
            self.transform.position -= forward * 4.0 * dt;
        }

        if input.keyboard.held(KeyCode::KeyD) {
            self.transform.position += right * 4.0 * dt;
        }

        if input.keyboard.held(KeyCode::Space) {
            self.transform.position.y += 4.0 * dt;
        }

        if input.keyboard.held(KeyCode::ShiftLeft) {
            self.transform.position.y -= 4.0 * dt;
        }

        let (mouse_x, mouse_y) = input.mouse.mouse_diff();

        let norm_mouse_x = mouse_x * 0.004;
        let norm_mouse_y = mouse_y * 0.004;

        #[cfg(not(feature = "capture_mouse"))]
        if input.mouse.held(MouseButton::Left) {
            self.transform.yaw += norm_mouse_x;
            self.transform.pitch -= norm_mouse_y;
        }
        #[cfg(feature = "capture_mouse")]
        {
            self.transform.yaw += norm_mouse_x;
            self.transform.pitch -= norm_mouse_y;
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

    pub fn as_camera_data(self) -> Data {
        let view_proj =
            self.projection.build_projection_matrix() * self.transform.build_translation_matrix();
        let view_pos = self.transform.position; // glam::Vec4::from((self.transform.position, 8008135_f32)); // :3

        Data {
            view_pos,
            view_proj,
        }
    }
}
