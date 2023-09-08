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
mod textures;
pub use textures::Textures;

pub struct Object {
    pub transform: render::Transform,
    pub mesh: render::Mesh,
    pub textures: textures::Textures,
}

pub struct Prepared<'obj> {
    mesh: render::PreparedMesh,
    textures: &'obj textures::Textures,

    transform_index: i32,
}

impl Object {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader) -> Self {
        let transform = render::Transform::new();
        // let transform =
        //     render::Transform::from_xyz((i % 20 - 10) as f32 * 2., 0.0, (i / 20 - 10) as f32 * 2.);

        let (_, models) = assets.models.load("assets/meshes/cube.obj");
        let mesh = render::Mesh::from_tobj_mesh(&models[0].mesh);

        let albedo_id = assets
            .textures
            .load(render_state, "assets/textures/cube-diffuse.jpg");
        let normal_id = assets
            .textures
            .load(render_state, "assets/textures/cube-normal.png");

        let textures = Textures::new(
            render_state,
            assets.textures.get_expect(albedo_id),
            assets.textures.get_expect(normal_id),
        );

        Self {
            transform,
            mesh,
            textures,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.transform.rotation *= glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 1.0 * dt, 0.0);
    }

    pub fn prepare(&self, resources: &mut scene::PrepareResources<'_>) -> Prepared<'_> {
        let transform_index = resources.transform.push(&self.transform) as i32;
        let mesh = self.mesh.prepare(resources);
        Prepared {
            mesh,
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
            render_pass.set_bind_group(1, resources.transform, &[]);

            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&self.transform_index),
            );

            self.textures.bind(render_pass, 2);
            self.mesh.draw(resources, render_pass);
        }

        render_pass.pop_debug_group();
    }
}
