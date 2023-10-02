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

pub struct Loader {
    pub textures: assets::Textures,
    pub models: assets::Models,
    pub gltf: assets::Gltf,
}

impl Loader {
    pub fn new() -> Self {
        let textures = assets::textures::Textures::new();
        let models = assets::Models::new();
        let gltf = assets::Gltf::new();

        Self {
            textures,
            models,
            gltf,
        }
    }

    pub fn load_gltf(&mut self, path: impl AsRef<camino::Utf8Path>) {
        let gltf_id = self.gltf.load(path);

        let gltf::Gltf { document, blob } = self.gltf.get_expect(gltf_id).clone();
        let buffers =
            gltf::import_buffers(&document, None, blob).expect("failed to import gltf buffers");
        let images =
            gltf::import_images(&document, None, &buffers).expect("failed to import gltf images");

        // FIXME: handle samplers and textures better?
        for texture in document.textures() {
            let texture_id = texture.index();

            let image = &images[texture.source().index()];
        }
    }
}
