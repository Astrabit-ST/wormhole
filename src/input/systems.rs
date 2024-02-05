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

use crate::input;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct InputSystem;

pub fn keyboard(
    mut input_state: ResMut<'_, input::State>,
    mut keyboard_events: EventReader<input::KeyboardEvent>,
) {
    input_state.bypass_change_detection().keyboard.start_frame();
    for &event in keyboard_events.read() {
        input_state.keyboard.process(event);
    }
}

pub fn mouse(
    mut input_state: ResMut<'_, input::State>,
    mut mouse_button_events: EventReader<input::MouseButtonEvent>,
    mut mouse_wheel_events: EventReader<input::MouseWheel>,
    mut mouse_motion_events: EventReader<input::MouseMotion>,
) {
    input_state.bypass_change_detection().mouse.start_frame();
    for &event in mouse_button_events.read() {
        input_state.mouse.process_button(event)
    }
    for &event in mouse_wheel_events.read() {
        input_state.mouse.process_mouse_wheel(event)
    }
    for &event in mouse_motion_events.read() {
        input_state.mouse.process_mouse_motion(event)
    }
}

pub fn close_requested(
    mut input_state: ResMut<'_, input::State>,
    mut close_requested: EventReader<input::CloseRequested>,
) {
    input_state
        .bypass_change_detection()
        .reset_close_requested();
    if close_requested.read().last().is_some() {
        input_state.process_close_requsted()
    }
}

pub fn window_resize(
    mut input_state: ResMut<'_, input::State>,
    mut window_resize_events: EventReader<input::WindowResized>,
) {
    input_state.bypass_change_detection().reset_window_size();
    for &event in window_resize_events.read() {
        input_state.process_resize(event)
    }
}
