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
#![warn(clippy::suspicious, clippy::perf, clippy::style)]
#![allow(clippy::new_without_default)]

pub mod assets {
    mod loader;
    pub use loader::Loader;

    mod gltf;
    pub use gltf::File as GltfFile;
    pub use gltf::Gltf;
    pub use gltf::Id as GltfId;

    mod textures;
    pub use textures::Id as TextureId;
    pub use textures::Textures;

    mod models;
    pub use models::Id as ModelId;
    pub use models::Model;
    pub use models::Models;

    mod materials;
    pub use materials::Id as MaterialId;
    pub use materials::Materials;

    use crate::render;
    use crate::scene;

    pub fn init_into(render_state: &render::State, builder: &mut scene::WorldBuilder) {
        builder.insert_resource(Loader::new(render_state));
    }
}

pub mod components {
    mod transform;
    pub use transform::Transform;

    pub mod light;
    pub use light::Light;

    pub mod mesh_renderer;
    pub use mesh_renderer::MeshRenderer;

    mod camera;
    pub use camera::Camera;
}

pub mod input {
    mod keyboard;
    pub use keyboard::Keyboard;

    mod events;
    pub use events::*;

    mod mouse;
    pub use mouse::Mouse;

    mod controller;
    pub use controller::Controller;

    mod state;
    pub use state::State;

    mod systems;

    use crate::scene;
    use bevy_ecs::prelude::*;

    pub fn init_into(builder: &mut scene::WorldBuilder) {
        builder
            .insert_resource(State::new())
            .add_event::<KeyboardEvent>()
            .add_event::<KeyboardEvent>()
            .add_event::<MouseButtonEvent>()
            .add_event::<MouseWheel>()
            .add_event::<MouseMotion>()
            .add_event::<WindowResized>()
            .add_event::<CloseRequested>()
            .add_event::<Exit>()
            .add_systems(
                scene::PreUpdate,
                (
                    systems::keyboard,
                    systems::mouse,
                    systems::close_requested,
                    systems::window_resize,
                )
                    .in_set(systems::InputSystem),
            );
    }
}

pub mod physics {
    pub mod state;
    pub use state::State;

    mod components {
        mod rigid_body;
        pub use rigid_body::RigidBody;
    }
    pub use components::*;

    pub mod systems;

    use crate::scene;
    use bevy_ecs::prelude::*;

    pub fn init_into(builder: &mut scene::WorldBuilder) {
        builder
            .insert_resource(State::new())
            .configure_sets(
                scene::FixedUpdate,
                (systems::SyncData, systems::Step, systems::WriteBack).chain(),
            )
            .add_systems(scene::FixedUpdate, (systems::step).in_set(systems::Step))
            .add_systems(
                scene::FixedUpdate,
                (systems::write_back_rigid_bodies).in_set(systems::WriteBack),
            );
    }
}

pub mod render {
    pub mod buffer {
        pub mod dynamic;

        pub mod single;

        pub mod geometry;

        pub mod instances;
    }

    pub mod binding_helpers;
    pub use binding_helpers::{BindGroupBuilder, BindGroupLayoutBuilder};

    mod color;
    pub use color::Color;

    mod instance;
    pub use instance::MeshInstance;

    mod mesh;
    pub use mesh::Mesh;
    pub use mesh::VertexFormat;

    pub mod state;
    pub use state::State;

    pub mod texture;
    pub use texture::Texture;
    pub use texture::TextureFormat;

    pub mod traits;

    pub mod material;
    pub use material::Material;

    pub mod system;

    use crate::scene;

    pub fn init_into(render_state: State, builder: &mut scene::WorldBuilder) {
        builder
            .insert_resource(render_state)
            .add_systems(scene::Update, system::render);
    }
}

pub mod scene;

pub mod shaders {
    pub mod light;
    pub mod object;
}

pub mod time;

pub mod player;
