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
use std::time::Instant;

use super::{Fixed, Real, Time, Virtual};
use crate::scene;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct TimeSystem;

pub fn update_time(mut time: ResMut<Time<Real>>) {
    let new_time = Instant::now();
    time.update_with_instant(new_time);
}

pub fn update_virtual_time(
    mut current: ResMut<Time>,
    mut virt: ResMut<Time<Virtual>>,
    real: Res<Time<Real>>,
) {
    let raw_delta = real.delta();
    virt.advance_with_raw_delta(raw_delta);
    *current = virt.as_generic();
}

pub fn run_fixed_main_schedule(world: &mut World) {
    let delta = world.resource::<Time<Virtual>>().delta();
    world.resource_mut::<Time<Fixed>>().accumulate(delta);

    let _ = world.try_schedule_scope(scene::FixedMain, |world, schedule| {
        while world.resource_mut::<Time<Fixed>>().expend() {
            *world.resource_mut::<Time>() = world.resource::<Time<Fixed>>().as_generic();
            schedule.run(world);
        }
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}
