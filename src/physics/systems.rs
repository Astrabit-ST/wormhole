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

use crate::components;
use crate::physics;
use crate::time;

#[derive(SystemSet, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct SyncData;

#[derive(SystemSet, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct Step;

#[derive(SystemSet, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct WriteBack;

pub fn step(mut physics_state: ResMut<'_, physics::State>, time: Res<'_, time::Time<time::Fixed>>) {
    physics_state.integration_parameters.dt = time.delta_seconds();
    physics_state.step();
}

pub fn write_back_rigid_bodies(
    physics_state: ResMut<'_, physics::State>,
    mut query: Query<&mut components::Transform, With<physics::RigidBody>>,
) {
    for (_, rigid_body) in physics_state.rigid_body_set.iter() {
        let entity = Entity::from_bits(rigid_body.user_data as u64);
        if let Ok(mut transform) = query.get_mut(entity) {
            let translation = rigid_body.translation();
            let rotation = rigid_body.rotation();
            transform.position = glam::vec3(translation.x, translation.y, translation.z);
            transform.rotation = glam::quat(rotation.i, rotation.j, rotation.k, rotation.w);
        }
    }
}
