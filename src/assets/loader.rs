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
use crate::assets;
use crate::render;

pub struct Loader {
    pub textures: assets::Textures,
    pub models: assets::Models,
    pub materials: assets::Materials,
    pub gltf: assets::Gltf,
}

impl Loader {
    pub fn new(render_state: &render::State) -> Self {
        let textures = assets::textures::Textures::new(render_state);
        let models = assets::Models::new();
        let materials = assets::Materials::new();
        let gltf = assets::Gltf::new();

        Self {
            textures,
            models,
            materials,
            gltf,
        }
    }

    pub fn load_gltf(&mut self, render_state: &render::State, path: impl AsRef<camino::Utf8Path>) {
        let gltf_id = self.gltf.load(path);

        let assets::GltfFile {
            document,
            buffers,
            images,
        } = self.gltf.get_expect(gltf_id);

        // FIXME: avoid recreating images multiple times?
        for texture in document.textures() {
            let texture_id = texture.index();
            let texture = render::Texture::from_gltf(render_state, texture, images);
            self.textures
                .insert(assets::TextureId::Gltf(gltf_id, texture_id), texture);
        }

        for material in document.materials() {
            let material_id = material.index().unwrap_or_default();
            let material =
                render::Material::from_gltf(render_state, gltf_id, &self.textures, material);
            self.materials
                .insert(assets::MaterialId::Gltf(gltf_id, material_id), material);
        }

        for mesh in document.meshes() {
            let mesh_id = mesh.index();
            let model = assets::Model::from_gltf(gltf_id, mesh, buffers);
            self.models
                .insert(assets::ModelId::Gltf(gltf_id, mesh_id), model);
        }
    }
}
