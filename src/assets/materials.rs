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

use itertools::Itertools;
use wgpu::util::DeviceExt;

pub struct Materials {
    pub(super) materials: indexmap::IndexMap<Id, render::Material>,
    buffer: Option<wgpu::Buffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    // Path id
    Path(u64),
    // Gltf id, mesh id
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

impl Materials {
    pub(super) fn new() -> Self {
        Self {
            materials: indexmap::IndexMap::new(),
            buffer: None,
        }
    }

    pub fn insert(&mut self, id: Id, material: render::Material) -> Option<render::Material> {
        self.buffer.take();
        self.materials.insert(id, material)
    }

    pub fn get_expect(&self, id: Id) -> &render::Material {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&render::Material> {
        self.materials.get(&id)
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.buffer.take();
        self.materials.retain(|i, _| ids.contains(i))
    }

    pub fn id_to_bindgroup_index(&self, id: Id) -> Option<usize> {
        self.materials.get_index_of(&id).map(|i| i + 1) // add 1 because 0 is the "null" id
    }
}

impl Materials {
    pub fn get_or_update_buffer(
        &mut self,
        render_state: &render::State,
        textures: &assets::Textures,
    ) -> &wgpu::Buffer {
        if self.buffer.is_none() {
            self.buffer = Some(self.create_buffer(render_state, textures));
        }
        self.buffer.as_ref().unwrap()
    }

    fn create_buffer(
        &self,
        render_state: &render::State,
        textures: &assets::Textures,
    ) -> wgpu::Buffer {
        let data = std::iter::once(&render::Material::default())
            .chain(self.materials.values())
            .map(|m| m.as_data(textures))
            .collect_vec();

        render_state
            .wgpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("wormhole material buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::STORAGE,
            })
    }
}
