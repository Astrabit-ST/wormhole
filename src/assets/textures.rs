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
use std::collections::HashMap;

use crate::render;

pub struct Textures {
    textures: slab::Slab<render::Texture>,
    ids: HashMap<camino::Utf8PathBuf, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(usize);

impl Textures {
    pub(super) fn new() -> Self {
        Self {
            textures: slab::Slab::new(),
            ids: HashMap::new(),
        }
    }

    pub fn load(&mut self, render_state: &render::State, path: impl AsRef<camino::Utf8Path>) -> Id {
        let path = path.as_ref();

        let id = self.ids.entry(path.to_path_buf()).or_insert_with(|| {
            let image = image::open(path).expect("failed to load texture");
            let texture =
                render::Texture::from_image(render_state, &image, render::TextureFormat::GENERIC);

            self.textures.insert(texture)
        });

        Id(*id)
    }

    pub fn load_with_format(
        &mut self,
        render_state: &render::State,
        path: impl AsRef<camino::Utf8Path>,
        format: render::TextureFormat,
    ) -> Id {
        let path = path.as_ref();

        let id = self.ids.entry(path.to_path_buf()).or_insert_with(|| {
            let image = image::open(path).expect("failed to load texture");
            let texture = render::Texture::from_image(render_state, &image, format);

            self.textures.insert(texture)
        });

        Id(*id)
    }

    pub fn get_expect(&self, id: Id) -> &render::Texture {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&render::Texture> {
        self.textures.get(id.0)
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.textures.retain(|i, _| ids.contains(&Id(i)))
    }
}
