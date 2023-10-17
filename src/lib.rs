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
#![warn(
    rust_2018_idioms,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style
)]
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
}

pub mod input {
    mod keyboard;
    pub use keyboard::Keyboard;

    mod mouse;
    pub use mouse::Mouse;

    mod controller;
    pub use controller::Controller;

    mod state;
    pub use state::State;
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

    mod camera;
    pub use camera::Camera;

    mod color;
    pub use color::Color;

    mod instance;
    pub use instance::Instance;

    mod mesh;
    pub use mesh::Mesh;
    pub use mesh::VertexFormat;

    pub mod state;
    pub use state::State;

    pub mod texture;
    pub use texture::Texture;
    pub use texture::TextureFormat;

    pub mod traits;

    mod transform;
    pub use transform::Transform;

    pub mod material;
    pub use material::Material;

    pub mod light;
    pub use light::Light;

    pub mod object;
    pub use object::Object;
}

pub mod scene;
