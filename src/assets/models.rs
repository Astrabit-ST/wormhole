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
use std::sync::Arc;

use crate::assets;
use crate::render;

pub struct Models {
    pub(super) models: HashMap<Id, Model>,
}

pub struct Model {
    pub name: String,
    pub meshes: Vec<Arc<render::Mesh>>,
}

impl Model {
    pub fn from_gltf(
        gltf_id: assets::GltfId,
        mesh: gltf::Mesh<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Self {
        let name = mesh.name().unwrap_or("unamed model").to_string();
        let meshes = mesh
            .primitives()
            .map(|primitive| render::Mesh::from_gltf_primitive(gltf_id, primitive, buffers))
            .map(Arc::new)
            .collect();
        Self { name, meshes }
    }
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

impl Models {
    pub(super) fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: Id, model: Model) -> Option<Model> {
        self.models.insert(id, model)
    }

    pub fn load_tobj(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        material_id: assets::MaterialId,
    ) -> Id {
        self.load_tobj_with_options(path, &tobj::GPU_LOAD_OPTIONS, material_id)
    }

    pub fn load_tobj_with_options(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        load_options: &tobj::LoadOptions,
        material_id: assets::MaterialId,
    ) -> Id {
        let path = path.as_ref();
        let id = Id::from_path(path);

        self.models.entry(id).or_insert_with(|| {
            // FIXME: this behavior is probably wrong.
            let (meshes, _) = tobj::load_obj(path, load_options).expect("failed to load models");
            let meshes = meshes
                .into_iter()
                .map(|m| render::Mesh::from_tobj_mesh(m.mesh, material_id))
                .map(Arc::new)
                .collect();

            Model {
                meshes,
                name: path.to_string(),
            }
        });

        id
    }

    pub fn get_expect(&self, id: Id) -> &Model {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&Model> {
        self.models.get(&id)
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.models.retain(|i, _| ids.contains(i))
    }
}
