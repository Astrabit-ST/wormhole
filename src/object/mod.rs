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

mod shader;
pub use shader::Shader;
mod textures;
pub use textures::Textures;

pub struct Object {
    pub transform: render::Transform,
    pub mesh: render::Mesh,
    pub textures: textures::Textures,
}

pub struct Prepared<'obj> {
    mesh: &'obj render::Mesh,
    textures: &'obj textures::Textures,

    transform_index: u32,
}

impl Object {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader, i: usize) -> Self {
        let transform = render::Transform::from_position(glam::Vec3::new(
            (i % 10) as f32 * 4.,
            0.,
            (i / 10) as f32 * 4.,
        ));

        let (models, _) = tobj::load_obj(
            "assets/meshes/cube.obj",
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .expect("failed to load models");
        let mesh = render::Mesh::from_tobj_mesh(render_state, &models[0].mesh);

        let (_, albedo) = assets
            .textures
            .load(render_state, "assets/textures/cube-diffuse.jpg");

        let textures = Textures::new(render_state, albedo);

        Self {
            transform,
            mesh,
            textures,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.transform.rotation *=
            glam::Quat::from_euler(glam::EulerRot::XYZ, 180_f32.to_radians() * dt, 0., 0.);
    }

    pub fn prepare(&self, resources: &mut scene::PrepareResources<'_>) -> Prepared<'_> {
        let transform_index = resources
            .transform
            .push(&self.transform)
            .expect("failed to write transform data") as u32;
        Prepared {
            mesh: &self.mesh,
            textures: &self.textures,
            transform_index,
        }
    }
}

impl<'obj> Prepared<'obj> {
    pub fn draw(
        self,
        resources: &scene::RenderResources<'obj>,
        render_pass: &mut wgpu::RenderPass<'obj>,
    ) {
        Shader::bind(render_pass);

        render_pass.set_bind_group(0, resources.camera, &[]);
        render_pass.set_bind_group(1, resources.transform, &[]);

        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::bytes_of(&self.transform_index),
        );

        self.textures.bind(render_pass, 2);
        self.mesh.draw(render_pass);
    }
}
