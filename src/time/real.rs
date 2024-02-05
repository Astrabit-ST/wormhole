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

use std::time::{Duration, Instant};

use super::Time;

#[derive(Debug, Clone, Copy)]
pub struct Real {
    startup: Instant,
    first_update: Option<Instant>,
    last_update: Option<Instant>,
}

impl Default for Real {
    fn default() -> Self {
        Self {
            startup: Instant::now(),
            first_update: None,
            last_update: None,
        }
    }
}

impl Time<Real> {
    pub fn new(startup: Instant) -> Self {
        Self::new_with(Real {
            startup,
            ..Default::default()
        })
    }

    pub fn update(&mut self) {
        let instant = Instant::now();
        self.update_with_instant(instant)
    }

    pub fn update_with_duration(&mut self, duration: Duration) {
        let last_update = self.context().last_update.unwrap_or(self.context().startup);
        self.update_with_instant(last_update + duration)
    }

    pub fn update_with_instant(&mut self, instant: Instant) {
        let Some(last_update) = self.context().last_update else {
            let context = self.context_mut();
            context.first_update = Some(instant);
            context.last_update = Some(instant);
            return;
        };
        let delta = instant - last_update;
        self.advance_by(delta);
        self.context_mut().last_update = Some(last_update);
    }

    pub fn startup(&self) -> Instant {
        self.context().startup
    }

    pub fn first_update(&self) -> Option<Instant> {
        self.context().first_update
    }

    pub fn last_update(&self) -> Option<Instant> {
        self.context().last_update
    }
}
