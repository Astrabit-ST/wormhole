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
use std::time::Duration;

use crate::scene;

pub mod fixed;
pub use fixed::Fixed;

pub mod real;
pub use real::Real;

pub mod virt;
pub use virt::Virtual;

mod systems;

#[derive(Resource, Debug, Clone, Copy)]
pub struct Time<T: Default = ()> {
    context: T,
    delta: Duration,
    elapsed: Duration,
}

impl<T: Default> Default for Time<T> {
    fn default() -> Self {
        Self {
            context: T::default(),
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }
}

impl<T: Default> Time<T> {
    pub fn new_with(context: T) -> Self {
        Self {
            context,
            ..Default::default()
        }
    }

    pub fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.elapsed += delta;
    }

    pub fn advance_to(&mut self, elapsed: Duration) {
        assert!(elapsed >= self.elapsed, "tried to move backwards in time");
        self.advance_by(elapsed - self.elapsed)
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta.as_secs_f64()
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    pub fn elapsed_seconds_f64(&self) -> f64 {
        self.elapsed.as_secs_f64()
    }

    pub fn context(&self) -> &T {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut T {
        &mut self.context
    }

    pub fn as_generic(&self) -> Time<()> {
        Time {
            context: (),
            delta: self.delta,
            elapsed: self.elapsed,
        }
    }
}

pub fn init_into(builder: &mut scene::WorldBuilder) {
    builder
        .init_resource::<Time>()
        .init_resource::<Time<Real>>()
        .init_resource::<Time<Virtual>>()
        .init_resource::<Time<Fixed>>()
        .init_resource::<bevy_ecs::event::EventUpdateSignal>()
        .add_systems(
            scene::First,
            (
                systems::update_time,
                systems::update_virtual_time.after(systems::update_time),
            )
                .in_set(systems::TimeSystem),
        )
        .add_systems(scene::RunFixedMainLoop, systems::run_fixed_main_schedule)
        .add_systems(
            scene::FixedPostUpdate,
            bevy_ecs::event::event_queue_update_system,
        );
}
