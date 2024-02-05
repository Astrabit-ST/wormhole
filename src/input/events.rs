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

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta},
    keyboard::PhysicalKey,
};

use bevy_ecs::prelude::*;

#[derive(bevy_ecs::system::SystemParam)]
pub struct EventWriters<'w> {
    pub keyboard: EventWriter<'w, KeyboardEvent>,
    pub mouse_button: EventWriter<'w, MouseButtonEvent>,
    pub mouse_motion: EventWriter<'w, MouseMotion>,
    pub mouse_wheel: EventWriter<'w, MouseWheel>,
    pub window_resized: EventWriter<'w, WindowResized>,
    pub close_requested: EventWriter<'w, CloseRequested>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[derive(Event)]
pub struct KeyboardEvent {
    pub key_code: PhysicalKey,
    pub state: ElementState,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[derive(Event)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub state: ElementState,
}

#[derive(PartialEq, Clone, Copy)]
#[derive(Event)]
pub struct MouseMotion {
    pub delta: glam::Vec2,
}

#[derive(PartialEq, Clone, Copy)]
#[derive(Event)]
pub struct MouseWheel {
    pub delta: MouseScrollDelta,
}

#[derive(PartialEq, Clone, Copy)]
#[derive(Event)]
pub struct WindowResized {
    pub size: PhysicalSize<u32>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[derive(Event)]
pub struct CloseRequested;

#[derive(PartialEq, Eq, Clone, Copy)]
#[derive(Event)]
pub struct Exit;
