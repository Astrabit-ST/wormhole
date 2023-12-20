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
use crate::render;
use crate::scene;

use bevy_ecs::prelude::*;

pub fn input(
    input_state: Res<input::State>,
    render_state: Res<render::State>,
    mut scene_buffers: ResMut<scene::Buffers>,
    mut camera: ResMut<render::Camera>,
) {
    if let Some(size) = input_state.new_window_size() {
        render_state.resize(size);
    }

    if input_state.new_window_size().is_some() {
        scene_buffers.gbuffer.resize_to_screen(&render_state);
    }

    camera.update(&input_state, 1.0 / 144.0) // fixme
}
