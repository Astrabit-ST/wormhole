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
use crate::player;
use crate::render;
use crate::scene;

use bevy_ecs::prelude::*;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub fn input(
    input_state: Res<input::State>,
    mut render_state: ResMut<render::State>,
    mut scene_buffers: ResMut<scene::Buffers>,
    mut player: ResMut<player::Player>,
) {
    if let Some(size) = input_state.new_window_size() {
        render_state.resize(size);
    }

    if let Some(size) = input_state.new_window_size() {
        scene_buffers.gbuffer.resize_to_screen(&render_state);
        player.camera.aspect = size.width as f32 / size.height as f32;
    }

    let forward = player.transform.forward();
    let left = player.transform.left();
    let dt = 1.0 / 144.0;

    if input_state.keyboard.held(KeyCode::KeyW) {
        player.transform.position += forward * 4.0 * dt;
    }

    if input_state.keyboard.held(KeyCode::KeyA) {
        player.transform.position += left * 4.0 * dt;
    }

    if input_state.keyboard.held(KeyCode::KeyS) {
        player.transform.position -= forward * 4.0 * dt;
    }

    if input_state.keyboard.held(KeyCode::KeyD) {
        player.transform.position -= left * 4.0 * dt;
    }

    if input_state.keyboard.held(KeyCode::Space) {
        player.transform.position.y += 4.0 * dt;
    }

    if input_state.keyboard.held(KeyCode::ShiftLeft) {
        player.transform.position.y -= 4.0 * dt;
    }

    let (mouse_x, mouse_y) = input_state.mouse.mouse_diff();

    let norm_mouse_x = mouse_x * 0.004;
    let norm_mouse_y = mouse_y * 0.004;

    #[cfg(not(feature = "capture_mouse"))]
    if input_state.mouse.held(MouseButton::Left) {
        player.transform.rotation *=
            glam::Quat::from_euler(glam::EulerRot::XYZ, norm_mouse_y, norm_mouse_x, 0.0);
    }
    #[cfg(feature = "capture_mouse")]
    {
        player.transform.rotation *=
            glam::Quat::from_euler(glam::EulerRot::XYZ, norm_mouse_y, norm_mouse_x, 0.0);
    }
}
