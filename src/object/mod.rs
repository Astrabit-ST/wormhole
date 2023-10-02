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
    transform_index: i32,
    mesh_indices: Vec<scene::MeshIndex>,
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
        let transform_index = resources.transforms.push(&self.transform) as i32;

        Prepared {
            transform_index,
            mesh_indices: self.mesh_indices.clone(),
        }
    }
}

impl Prepared {
    pub fn draw<'rpass>(
        self,
        resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        render_pass.push_debug_group("wormhole object draw");

        {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&self.transform_index),
            );

            for mesh_index in self.mesh_indices {
                mesh_index.draw(resources, render_pass);
            }
        }

        render_pass.pop_debug_group();
    }
}
