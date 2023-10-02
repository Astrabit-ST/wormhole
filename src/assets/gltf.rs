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

pub struct Gltf {
    documents: HashMap<Id, gltf::Gltf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    pub fn from_path(path: impl AsRef<camino::Utf8Path>) -> Self {
        use std::hash::{Hash, Hasher};

        let path = path.as_ref();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);

        Self(hasher.finish())
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

impl Gltf {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<camino::Utf8Path>) -> Id {
        let path = path.as_ref();
        let id = Id::from_path(path);

        self.documents
            .entry(id)
            .or_insert_with(|| gltf::Gltf::open(path).expect("failed to open gltf file"));

        id
    }

    pub fn get_expect(&self, id: Id) -> &gltf::Gltf {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&gltf::Gltf> {
        self.documents.get(&id)
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.documents.retain(|i, _| ids.contains(i))
    }
}
