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
use std::collections::HashSet;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use super::KeyboardEvent;

pub struct Keyboard {
    // Use a ping pong style buffer.
    // At the beginning of each frame, current and previous are swapped and current is cloned from previous.
    // This is pretty efficient for memory usage.
    current: HashSet<PhysicalKey>,
    previous: HashSet<PhysicalKey>,
}

impl Keyboard {
    pub fn new() -> Self {
        let current = HashSet::new();
        let previous = HashSet::new();

        Self { current, previous }
    }

    pub fn start_frame(&mut self) {
        // We swap the current and previous keypress buffers.
        // Then, current clones the data from previous. This avoids an allocation as is like a memcpy!
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clone_from(&self.previous);
    }

    pub fn process(&mut self, event: KeyboardEvent) {
        // Pattern matching my beloved

        match event.state {
            ElementState::Pressed => self.current.insert(event.key_code),
            ElementState::Released => self.current.remove(&event.key_code),
        };
    }

    pub fn pressed(&self, key: KeyCode) -> bool {
        self.current.contains(&PhysicalKey::Code(key))
            && !self.previous.contains(&PhysicalKey::Code(key))
    }

    pub fn released(&self, key: KeyCode) -> bool {
        self.previous.contains(&PhysicalKey::Code(key))
            && !self.current.contains(&PhysicalKey::Code(key))
    }

    pub fn held(&self, key: KeyCode) -> bool {
        self.current.contains(&PhysicalKey::Code(key))
    }
}
