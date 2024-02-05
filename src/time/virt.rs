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

use std::time::Duration;

use super::Time;

#[derive(Debug, Copy, Clone)]
pub struct Virtual {
    max_delta: Duration,
    paused: bool,
    relative_speed: f64,
    effective_speed: f64,
}

impl Default for Virtual {
    fn default() -> Self {
        Self {
            max_delta: Time::<Virtual>::DEFAULT_MAX_DELTA,
            paused: false,
            relative_speed: 1.0,
            effective_speed: 1.0,
        }
    }
}

impl Time<Virtual> {
    const DEFAULT_MAX_DELTA: Duration = Duration::from_millis(250);

    pub fn from_max_delta(max_delta: Duration) -> Self {
        let mut ret = Self::default();
        ret.set_max_delta(max_delta);
        ret
    }

    pub fn max_delta(&self) -> Duration {
        self.context().max_delta
    }

    pub fn set_max_delta(&mut self, max_delta: Duration) {
        assert_ne!(max_delta, Duration::ZERO, "tried to set max delta to zero");
        self.context_mut().max_delta = max_delta;
    }

    pub fn relative_speed(&self) -> f32 {
        self.relative_speed_f64() as f32
    }

    pub fn relative_speed_f64(&self) -> f64 {
        self.context().relative_speed
    }

    pub fn effective_speed(&self) -> f32 {
        self.context().effective_speed as f32
    }

    pub fn effective_speed_f64(&self) -> f64 {
        self.context().effective_speed
    }

    pub fn set_relative_speed(&mut self, ratio: f32) {
        self.set_relative_speed_f64(ratio as f64);
    }

    pub fn set_relative_speed_f64(&mut self, ratio: f64) {
        assert!(ratio.is_finite(), "tried to go infinitely fast");
        assert!(ratio >= 0.0, "tried to go back in time");
        self.context_mut().relative_speed = ratio;
    }

    pub fn pause(&mut self) {
        self.context_mut().paused = true;
    }

    pub fn unpause(&mut self) {
        self.context_mut().paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.context().paused
    }

    pub fn advance_with_raw_delta(&mut self, raw_delta: Duration) {
        let max_delta = self.context().max_delta;
        let clamped_delta = if raw_delta > max_delta {
            max_delta
        } else {
            raw_delta
        };
        let effective_speed = if self.context().paused {
            0.0
        } else {
            self.context().relative_speed
        };
        let delta = if effective_speed != 1.0 {
            clamped_delta.mul_f64(effective_speed)
        } else {
            // avoid rounding when at normal speed
            clamped_delta
        };
        self.context_mut().effective_speed = effective_speed;
        self.advance_by(delta);
    }
}
