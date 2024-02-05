// Copyright (C) 2024 Lily Lyons
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

use bevy_ecs::prelude::*;

mod systems;

use crate::{
    components, render,
    scene::{self, FixedUpdate},
};

#[derive(Resource)]
pub struct Player {
    pub camera: components::Camera,
    pub transform: components::Transform,
}

impl Player {
    pub fn new(render_state: &render::State) -> Self {
        let camera = components::Camera::new(render_state);
        let transform = components::Transform::from_position_rotation(
            glam::vec3(0.0, 0.0, 0.0),
            glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
        );

        Self { camera, transform }
    }
}

pub fn init_into(render_state: &render::State, builder: &mut scene::WorldBuilder) {
    builder
        .insert_resource(Player::new(render_state))
        .add_systems(FixedUpdate, systems::movement);
}
