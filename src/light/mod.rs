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

use rand::Rng;

mod shader;

pub struct Light {
    transform: render::Transform,
    mesh: render::Mesh,

    intensity: f32,
    range: f32,
    color: render::Color,
}

#[repr(u8)]
pub enum LightType {
    Point = 1,
}

pub struct PreparedObject {
    mesh: render::PreparedMesh,
    transform_index: i32,

    intensity: f32,
    color: render::Color,
}

pub struct PreparedLight {
    mesh: render::PreparedMesh,
    transform_index: i32,

    intensity: f32,
    range: f32,

    color: render::Color,
    position: glam::Vec3,
}

impl Light {
    pub fn new(assets: &mut assets::Loader) -> Self {
        let mut rng = rand::thread_rng();
        let transform = render::Transform::from_position_scale(
            glam::vec3(
                rng.gen_range(-10_f32..10_f32),
                rng.gen_range(-10_f32..10_f32),
                rng.gen_range(-10_f32..10_f32),
            ),
            glam::Vec3::splat(0.1),
        );

        let (_, models) = assets.models.load("assets/meshes/ico_sphere.obj");
        let mesh = render::Mesh::from_tobj_mesh(&models[0].mesh);

        let intensity = 1.0;
        let range = 15.0;
        let color = render::Color::from_rgb_normalized(glam::vec3(0.6, 0.65, 1.0));

        Light {
            transform,
            mesh,

            range,
            intensity,
            color,
        }
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn prepare_object(&self, resources: &mut scene::PrepareResources<'_>) -> PreparedObject {
        let transform_index = resources.transform.push(&self.transform) as i32;
        let mesh = self.mesh.prepare(resources);

        let intensity = self.intensity;
        let color = self.color;

        PreparedObject {
            mesh,
            transform_index,

            intensity,
            color,
        }
    }

    pub fn prepare_light(&self, resources: &mut scene::PrepareResources<'_>) -> PreparedLight {
        let transform = render::Transform {
            scale: glam::Vec3::splat(self.range),
            ..self.transform
        };

        let transform_index = resources.transform.push(&transform) as i32;
        let mesh = self.mesh.prepare(resources);

        let intensity = self.intensity;
        let range = self.range;
        let color = self.color;
        let position = self.transform.position;

        PreparedLight {
            mesh,
            transform_index,

            intensity,
            range,

            color,
            position,
        }
    }
}

impl PreparedObject {
    pub fn draw<'rpass>(
        self,
        resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable)]
        struct PushConstants {
            transform_index: i32,
            _pad: [u8; 8],

            intensity: f32,
            color: render::Color,
        }
        let push_constants = PushConstants {
            transform_index: self.transform_index,
            _pad: [0; 8],

            intensity: self.intensity,
            color: self.color,
        };
        let push_constants_bytes = bytemuck::bytes_of(&push_constants);

        render_pass.push_debug_group("wormhole light object draw");

        {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                &push_constants_bytes[0..4],
            );
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                12,
                &push_constants_bytes[12..],
            );

            self.mesh.draw(resources, render_pass);
        }

        render_pass.pop_debug_group();
    }
}

impl PreparedLight {
    pub fn draw<'rpass>(
        self,
        resources: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable)]
        struct PushConstants {
            transform_index: i32,

            intensity: f32,
            range: f32,

            _pad: [u8; 4],
            color: render::Color,
            position: glam::Vec3,
            _pad2: [u8; 4],
        }
        let push_constants = PushConstants {
            transform_index: self.transform_index,

            intensity: self.intensity,
            range: self.range,

            _pad: [0; 4],
            color: self.color,
            position: self.position,
            _pad2: [0; 4],
        };
        let push_constants_bytes = bytemuck::bytes_of(&push_constants);

        render_pass.push_debug_group("wormhole light draw");

        {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                &push_constants_bytes[0..4],
            );
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                8,
                &push_constants_bytes[8..],
            );

            self.mesh.draw(resources, render_pass);
        }

        render_pass.pop_debug_group();
    }
}
