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

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: render::Color,
    diffuse: render::Color,
    specular: render::Color,
}

#[repr(u8)]
pub enum LightType {
    Point = 1,
}

pub struct PreparedObject {
    mesh: render::PreparedMesh,
    transform_index: i32,

    color: render::Color,
}

pub struct PreparedLight {
    mesh: render::PreparedMesh,
    transform_index: i32,

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: render::Color,
    diffuse: render::Color,
    specular: render::Color,
    position: glam::Vec3,
}

impl Light {
    pub fn new(assets: &mut assets::Loader) -> Self {
        let mut rng = rand::thread_rng();
        let transform = render::Transform::from_position_scale(
            glam::vec3(
                rng.gen_range(-20_f32..20_f32),
                rng.gen_range(-20_f32..20_f32),
                rng.gen_range(-20_f32..20_f32),
            ),
            glam::Vec3::splat(0.1),
        );
        // let transform = render::Transform::from_position(glam::vec3(0.0, 5.0, 0.0));

        let (_, models) = assets.models.load("assets/meshes/ico_sphere.obj");
        let mesh = render::Mesh::from_tobj_mesh(&models[0].mesh);

        let constant = 1.0;
        let linear = 0.35;
        let quadratic = 0.44;

        let ambient = render::Color::from_rgb_normalized(glam::vec3(0.01, 0.01, 0.01));
        let diffuse = render::Color::from_rgb_normalized(glam::vec3(0.6, 0.65, 1.0));
        let specular = render::Color::from_rgb_normalized(glam::vec3(0.5, 0.5, 0.5));

        Light {
            transform,
            mesh,

            constant,
            linear,
            quadratic,

            ambient,
            diffuse,
            specular,
        }
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn prepare_object(&self, resources: &mut scene::PrepareResources<'_>) -> PreparedObject {
        let transform = render::Transform {
            scale: glam::Vec3::splat(0.1),
            ..self.transform
        };
        let transform_index = resources.transform.push(&transform) as i32;
        let mesh = self.mesh.prepare(resources);

        let color = self.diffuse;

        PreparedObject {
            mesh,
            transform_index,

            color,
        }
    }

    pub fn prepare_light(&self, resources: &mut scene::PrepareResources<'_>) -> PreparedLight {
        let light_max = self.diffuse.color.max_element();
        let radius = (-self.linear
            + (self.linear.powi(2)
                - 4. * self.quadratic * (self.constant - (256. / 5.) * light_max))
                .sqrt())
            / (2. * self.quadratic);
        let transform = render::Transform {
            scale: glam::Vec3::splat(radius * 2.),
            ..self.transform
        };

        let transform_index = resources.transform.push(&transform) as i32;
        let mesh = self.mesh.prepare(resources);

        let constant = self.constant;
        let linear = self.linear;
        let quadratic = self.quadratic;

        let ambient = self.ambient;
        let diffuse = self.diffuse;
        let specular = self.specular;
        let position = self.transform.position;

        PreparedLight {
            mesh,
            transform_index,

            constant,
            linear,
            quadratic,

            ambient,
            diffuse,
            specular,

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
            _pad: [u8; 12],

            color: render::Color,
        }
        let push_constants = PushConstants {
            transform_index: self.transform_index,
            _pad: [0; 12],

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

            constant: f32,
            linear: f32,
            quadratic: f32,

            ambient: render::Color,
            diffuse: render::Color,
            specular: render::Color,

            position: glam::Vec3,
            _pad: [u8; 4],
        }
        let push_constants = PushConstants {
            transform_index: self.transform_index,

            constant: self.constant,
            linear: self.linear,
            quadratic: self.quadratic,

            ambient: self.ambient,
            diffuse: self.diffuse,
            specular: self.specular,

            position: self.position,
            _pad: [0; 4],
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
