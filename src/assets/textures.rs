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

use crate::assets;
use crate::render;

pub struct Textures {
    pub(super) textures: HashMap<Id, render::Texture>,
    null_texture: render::Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    // Path id
    Path(u64),
    // Gltf id, texture id
    Gltf(assets::GltfId, usize),
}

impl Id {
    pub fn from_path(path: impl AsRef<camino::Utf8Path>) -> Self {
        use std::hash::{Hash, Hasher};

        let path = path.as_ref();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);

        Self::Path(hasher.finish())
    }

    pub fn from_gltf(gltf_id: assets::GltfId, texture_id: usize) -> Self {
        Self::Gltf(gltf_id, texture_id)
    }
}

impl<T> From<T> for Id
where
    T: AsRef<camino::Utf8Path>,
{
    fn from(value: T) -> Self {
        Self::from_path(value)
    }
}

impl Textures {
    pub(super) fn new(render_state: &render::State) -> Self {
        let null_texture = render::Texture::new(
            render_state,
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            render::TextureFormat::GENERIC,
        );
        Self {
            textures: HashMap::new(),
            null_texture,
        }
    }

    pub fn insert_image(
        &mut self,
        render_state: &render::State,
        id: Id,
        image: image::DynamicImage,
    ) -> Option<render::Texture> {
        let texture =
            render::Texture::from_image(render_state, &image, render::TextureFormat::GENERIC);
        self.insert(id, texture)
    }

    pub fn insert(&mut self, id: Id, texture: render::Texture) -> Option<render::Texture> {
        self.textures.insert(id, texture)
    }

    pub fn load_from_path(
        &mut self,
        render_state: &render::State,
        path: impl AsRef<camino::Utf8Path>,
    ) -> Id {
        self.load_from_path_with_format(render_state, path, render::TextureFormat::GENERIC)
    }

    pub fn load_from_path_with_format(
        &mut self,
        render_state: &render::State,
        path: impl AsRef<camino::Utf8Path>,
        format: render::TextureFormat,
    ) -> Id {
        let path = path.as_ref();
        let id = Id::from_path(path);

        self.textures.entry(id).or_insert_with(|| {
            let image = image::open(path).expect("failed to load texture");
            render::Texture::from_image(render_state, &image, format)
        });

        id
    }

    pub fn get_expect(&self, id: Id) -> &render::Texture {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&render::Texture> {
        self.textures.get(&id)
    }

    pub fn null_texture(&self) -> &render::Texture {
        &self.null_texture
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.textures.retain(|i, _| ids.contains(i))
    }
}
