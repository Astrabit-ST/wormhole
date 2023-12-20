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

use crate::render;
use crate::scene;

use bevy_ecs::prelude::*;

#[derive(Debug)]
#[derive(Component)]
pub struct MeshRenderer {
    pub mesh_index: scene::MeshIndex,
}

pub struct PreparedMesh {
    instance_index: u32,

    index_count: u32,
    index_offset: u32,
}

impl MeshRenderer {
    pub fn new(meshes: &mut scene::Meshes, mesh: std::sync::Arc<render::Mesh>) -> Self {
        let mesh_index = meshes.upload_mesh(mesh);
        Self { mesh_index }
    }

    pub fn prepare(
        &self,
        transform_index: u32,
        resources: &mut scene::PrepareResources<'_>,
    ) -> PreparedMesh {
        let instance = render::MeshInstance::from_mesh_transform_indices_with_materials(
            self.mesh_index,
            transform_index,
            &resources.assets.materials,
        );
        let instance_index = resources.instances.push(instance) as u32;
        PreparedMesh {
            instance_index,
            index_count: self.mesh_index.index_count as u32,
            index_offset: self.mesh_index.index_offset as u32,
        }
    }
}

impl PreparedMesh {
    pub fn draw(self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.push_debug_group("wormhole object draw");

        let index_start = self.index_offset / std::mem::size_of::<u32>() as u32;
        let index_end = index_start + self.index_count;

        let instance_start = self.instance_index;
        let instance_end = self.instance_index + 1;

        render_pass.draw_indexed(index_start..index_end, 0, instance_start..instance_end);

        render_pass.pop_debug_group();
    }
}
