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
pub struct Fixed {
    timestep: Duration,
    overstep: Duration,
}

impl Default for Fixed {
    fn default() -> Self {
        Self {
            timestep: Time::<Fixed>::DEFAULT_TIMESTEP,
            overstep: Duration::ZERO,
        }
    }
}

impl Time<Fixed> {
    const DEFAULT_TIMESTEP: Duration = Duration::from_micros(15625);

    pub fn from_duration(timestep: Duration) -> Self {
        let mut ret = Self::default();
        ret.set_timestep(timestep);
        ret
    }

    pub fn from_seconds(seconds: f64) -> Self {
        let mut ret = Self::default();
        ret.set_timestep_seconds(seconds);
        ret
    }

    pub fn from_hz(hz: f64) -> Self {
        let mut ret = Self::default();
        ret.set_timestep_hz(hz);
        ret
    }

    pub fn timestep(&self) -> Duration {
        self.context().timestep
    }

    pub fn set_timestep(&mut self, timestep: Duration) {
        assert_ne!(
            timestep,
            Duration::ZERO,
            "attempted to set fixed timestep to zero"
        );
        self.context_mut().timestep = timestep;
    }

    pub fn set_timestep_seconds(&mut self, seconds: f64) {
        assert!(
            seconds.is_sign_positive(),
            "seconds less than or equal to zero"
        );
        assert!(seconds.is_finite(), "seconds is infinite");
        self.set_timestep(Duration::from_secs_f64(seconds));
    }

    pub fn set_timestep_hz(&mut self, hz: f64) {
        assert!(hz.is_sign_positive(), "Hz less than or equal to zero");
        assert!(hz.is_finite(), "Hz is infinite");
        self.set_timestep_seconds(1.0 / hz);
    }

    pub fn overstep(&self) -> Duration {
        self.context().overstep
    }

    pub fn discard_overstep(&mut self, discard: Duration) {
        let context = self.context_mut();
        context.overstep = context.overstep.saturating_sub(discard);
    }

    pub fn overstep_fraction(&self) -> f32 {
        self.context().overstep.as_secs_f32() / self.context().timestep.as_secs_f32()
    }

    pub fn overstep_fraction_f64(&self) -> f64 {
        self.context().overstep.as_secs_f64() / self.context().timestep.as_secs_f64()
    }

    pub fn accumulate(&mut self, delta: Duration) {
        self.context_mut().overstep += delta;
    }

    pub fn expend(&mut self) -> bool {
        let timestep = self.timestep();
        if let Some(new_value) = self.context_mut().overstep.checked_sub(timestep) {
            self.context_mut().overstep = new_value;
            self.advance_by(timestep);
            true
        } else {
            false
        }
    }
}
