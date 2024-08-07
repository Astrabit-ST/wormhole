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
use rapier3d::prelude::*;

use crate::physics;

#[derive(Component, Debug)]
pub struct RigidBody {
    pub handle: RigidBodyHandle,
}

impl RigidBody {
    pub fn new(
        physics_state: &mut physics::State,
        entity: Entity,
        mut rigid_body: rapier3d::dynamics::RigidBody,
    ) -> Self {
        rigid_body.user_data = entity.to_bits() as u128;
        let handle = physics_state.rigid_body_set.insert(rigid_body);
        Self { handle }
    }
}
