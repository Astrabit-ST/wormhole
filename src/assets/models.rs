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

use crate::render;

pub struct Models {
    models: slab::Slab<Vec<Arc<render::Model>>>,
    ids: HashMap<camino::Utf8PathBuf, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(usize);

impl Models {
    pub(super) fn new() -> Self {
        Self {
            models: slab::Slab::new(),
            ids: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<camino::Utf8Path>) -> (Id, &[Arc<render::Model>]) {
        self.load_with_options(path, &tobj::GPU_LOAD_OPTIONS)
    }

    pub fn load_with_options(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        load_options: &tobj::LoadOptions,
    ) -> (Id, &[Arc<render::Model>]) {
        let path = path.as_ref();

        let id = self.ids.entry(path.to_path_buf()).or_insert_with(|| {
            let (models, _) = tobj::load_obj(path, load_options).expect("failed to load models");
            let models = models
                .into_iter()
                .map(render::Model::from_tobj_model)
                .map(Arc::new)
                .collect();

            self.models.insert(models)
        });
        let models = self
            .models
            .get(*id)
            .expect("asset not existent despite being loaded");

        (Id(*id), models)
    }

    pub fn get_expect(&self, id: Id) -> &[Arc<render::Model>] {
        self.get(id).expect("asset id nonexistent")
    }

    pub fn get(&self, id: Id) -> Option<&[Arc<render::Model>]> {
        self.models.get(id.0).map(Vec::as_slice)
    }

    pub fn keep_ids(&mut self, ids: &[Id]) {
        self.models.retain(|i, _| ids.contains(&Id(i)))
    }
}
