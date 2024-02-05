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
use crate::input;

use winit::keyboard::KeyCode;

use bevy_ecs::prelude::*;

use super::WindowResized;

#[derive(Resource)]
pub struct State {
    pub keyboard: input::Keyboard,
    pub mouse: input::Mouse,
    pub controller: input::Controller,

    close_requested: bool,
    new_size: Option<winit::dpi::PhysicalSize<u32>>,
}

impl State {
    pub fn new() -> Self {
        let keyboard = input::Keyboard::new();
        let mouse = input::Mouse::new();
        let controller = input::Controller::new();

        Self {
            keyboard,
            mouse,
            controller,

            close_requested: false,
            new_size: None,
        }
    }

    // notify everything that we're starting a new frame and set frame specific variables.
    pub fn start_frame(&mut self) {
        self.keyboard.start_frame();
        self.mouse.start_frame();

        self.new_size = None;
        self.close_requested = false;
    }

    pub fn process_resize(&mut self, resize: WindowResized) {
        self.new_size = Some(resize.size);
    }

    pub fn process_close_requsted(&mut self) {
        self.close_requested = true;
    }

    pub fn move_direction(&self) -> glam::Vec2 {
        let mut vector = glam::Vec2::ZERO;

        if self.keyboard.held(KeyCode::KeyW) {
            vector.y += 1.0;
        }

        if self.keyboard.held(KeyCode::KeyA) {
            vector.x -= 1.0;
        }

        if self.keyboard.held(KeyCode::KeyS) {
            vector.y -= 1.0;
        }

        if self.keyboard.held(KeyCode::KeyD) {
            vector.x -= 1.0;
        }

        vector
    }

    pub fn reset_close_requested(&mut self) {
        self.close_requested = false;
    }

    pub fn reset_window_size(&mut self) {
        self.new_size = None
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub fn new_window_size(&self) -> Option<winit::dpi::PhysicalSize<u32>> {
        self.new_size
    }
}
