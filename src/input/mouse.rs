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
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};

pub struct Mouse {
    mouse_diff: Option<(f32, f32)>,
    scroll_diff: (f32, f32),

    current: Properties,
    previous: Properties,
}

#[derive(Clone, Copy, Default)]
struct Properties {
    left: bool,
    right: bool,
    middle: bool,

    cursor: Option<(f32, f32)>,
}

// I just took this from some library that took it from three-rs, no idea why this magic number was chosen ¯\_(ツ)_/¯
const PIXELS_PER_LINE: f32 = 38.0;

impl Mouse {
    pub fn new() -> Self {
        Self {
            mouse_diff: None,
            scroll_diff: (0., 0.),

            current: Properties::default(),
            previous: Properties::default(),
        }
    }

    pub fn start_frame(&mut self) {
        self.mouse_diff = None;
        self.scroll_diff = (0.0, 0.0);

        self.previous = self.current;
        self.current.cursor = None;
    }

    pub fn process_window(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.current.cursor = Some((position.x as f32, position.y as f32));
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => self.current.left = matches!(state, ElementState::Pressed),
                MouseButton::Right => self.current.right = matches!(state, ElementState::Pressed),
                MouseButton::Middle => self.current.middle = matches!(state, ElementState::Pressed),
                MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => {} // todo maybe handle
            },
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => self.scroll_diff = (*x, *y),
                MouseScrollDelta::PixelDelta(delta) => {
                    self.scroll_diff = (
                        delta.x as f32 / PIXELS_PER_LINE,
                        delta.y as f32 / PIXELS_PER_LINE,
                    )
                }
            },
            _ => {}
        }
    }

    pub fn process_device(&mut self, event: &DeviceEvent) {
        match event {
            // This gives us raw deltas without the OS meddling, which is perfect for a first person camera
            // winit_input_helper lacks this, which is why I made this input handler
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_diff = Some((delta.0 as f32, delta.1 as f32))
            }
            DeviceEvent::MouseWheel { delta } => match delta {
                MouseScrollDelta::LineDelta(x, y) => self.scroll_diff = (*x, *y),
                MouseScrollDelta::PixelDelta(delta) => {
                    self.scroll_diff = (
                        delta.x as f32 / PIXELS_PER_LINE,
                        delta.y as f32 / PIXELS_PER_LINE,
                    )
                }
            },
            _ => {}
        }
    }

    pub fn mouse_diff(&self) -> (f32, f32) {
        self.mouse_diff.unwrap_or_default()
    }

    pub fn cursor_diff(&self) -> (f32, f32) {
        let current = self.current.cursor.unwrap_or_default();
        let previous = self.previous.cursor.unwrap_or_default();

        (current.0 - previous.0, current.1 - previous.1)
    }

    pub fn pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.current.left && !self.previous.left,
            MouseButton::Right => self.current.right && !self.previous.right,
            MouseButton::Middle => self.current.middle && !self.previous.middle,
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
        }
    }

    pub fn released(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => !self.current.left && self.previous.left,
            MouseButton::Right => !self.current.right && self.previous.right,
            MouseButton::Middle => !self.current.middle && self.previous.middle,
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
        }
    }

    pub fn held(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.current.left,
            MouseButton::Right => self.current.right,
            MouseButton::Middle => self.current.middle,
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
        }
    }
}
