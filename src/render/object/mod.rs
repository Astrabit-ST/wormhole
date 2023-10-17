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
use crate::scene;

pub mod shader;

pub struct Object {
    pub transform: render::Transform,
    pub mesh_indices: Vec<scene::MeshIndex>,
}

pub struct Prepared {
    meshes: Vec<PreparedMesh>,
}

pub struct PreparedMesh {
    instance_index: u32,

    index_count: u32,
    index_offset: u32,
}

impl Object {
    pub fn new(
        meshes: &mut scene::Meshes,
        transform: render::Transform,
        model: &assets::Model,
    ) -> Self {
        let mesh_indices = model
            .meshes
            .iter()
            .cloned()
            .map(|m| meshes.upload_mesh(m))
            .collect();
        Self {
            transform,
            mesh_indices,
        }
    }

    pub fn prepare(&self, resources: &mut scene::PrepareResources<'_>) -> Prepared {
        let transform_index = resources.transforms.push(&self.transform) as u32;

        Prepared {
            meshes: self
                .mesh_indices
                .iter()
                .copied()
                .map(|mesh_index| {
                    let instance =
                        render::Instance::from_mesh_transform_indices(mesh_index, transform_index);
                    let instance_index = resources.instances.push(instance) as u32;
                    PreparedMesh {
                        instance_index,
                        index_count: mesh_index.index_count as u32,
                        index_offset: mesh_index.index_offset as u32,
                    }
                })
                .collect(),
        }
    }
}

impl Prepared {
    pub fn draw(self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.push_debug_group("wormhole object draw");

        {
            for mesh in self.meshes {
                let index_start = mesh.index_offset / std::mem::size_of::<u32>() as u32;
                let index_end = index_start + mesh.index_count;

                let instance_start = mesh.instance_index;
                let instance_end = mesh.instance_index + 1;

                render_pass.draw_indexed(index_start..index_end, 0, instance_start..instance_end);
            }
        }

        render_pass.pop_debug_group();
    }
}
