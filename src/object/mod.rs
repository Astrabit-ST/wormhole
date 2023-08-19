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

mod shader;
pub use shader::Shader;

pub struct Object {
    pub transform: render::Transform,
    pub mesh: render::Mesh,
}

impl Object {
    pub fn new(render_state: &render::State) -> Self {
        let transform = render::Transform::new(render_state);
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

        Self { transform, mesh }
    }

    pub fn draw<'rpass>(
        &'rpass self,
        camera: &'rpass render::Camera,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        Shader::bind(render_pass);
        camera.bind(render_pass, 0);
        self.transform.bind(render_pass, 1);
        self.mesh.draw(render_pass);
    }
}
