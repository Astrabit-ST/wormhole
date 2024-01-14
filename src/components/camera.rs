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
use crate::{components, render};

use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy)]
#[derive(Component)]
pub struct Camera {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

pub struct Data {
    pub view_pos: glam::Vec3,
    pub view_proj: glam::Mat4,
}

impl Camera {
    pub fn new(render_state: &render::State) -> Self {
        let aspect = render_state.wgpu.surface_config.width as f32
            / render_state.wgpu.surface_config.height as f32;
        Camera {
            aspect,
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn as_camera_data(self, transform: components::Transform) -> Data {
        let projection_matrix =
            glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        let transform_matrix =
            glam::Mat4::look_to_rh(transform.position, transform.forward(), glam::Vec3::Y);
        let view_proj = projection_matrix * transform_matrix;
        let view_pos = transform.position; // glam::Vec4::from((self.transform.position, 8008135_f32)); // :3

        Data {
            view_pos,
            view_proj,
        }
    }
}
