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

pub mod shader;
mod textures;
pub use textures::Textures;

pub struct Object {
    pub transform: render::Transform,
    pub mesh_index: scene::MeshIndex,
    pub textures: textures::Textures,
}

pub struct Prepared<'obj> {
    model_index: scene::MeshIndex,
    textures: &'obj textures::Textures,

    transform_index: i32,
}

impl Object {
    pub fn new(
        transform: render::Transform,
        model_index: scene::MeshIndex,
        textures: textures::Textures,
    ) -> Self {
        Self {
            transform,
            mesh_index: model_index,
            textures,
        }
    }

    pub fn prepare(&self, resources: &mut scene::PrepareResources<'_>) -> Prepared<'_> {
        let transform_index = resources.transforms.push(&self.transform) as i32;

        Prepared {
            model_index: self.mesh_index,
            transform_index,
            textures: &self.textures,
        }
    }
}

impl<'obj> Prepared<'obj> {
    pub fn draw(
        self,
        resources: &scene::RenderResources<'obj>,
        render_pass: &mut wgpu::RenderPass<'obj>,
    ) {
        render_pass.push_debug_group("wormhole object draw");

        {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&self.transform_index),
            );

            self.textures.bind(render_pass, 2);
            self.model_index.draw(resources, render_pass);
        }

        render_pass.pop_debug_group();
    }
}
