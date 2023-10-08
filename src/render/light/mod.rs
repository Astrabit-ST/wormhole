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

pub struct Light {
    transform: render::Transform,
    mesh_index: scene::MeshIndex,

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
    instance_index: u32,
    index_count: u32,
    index_offset: u32,

    color: render::Color,
}

#[derive(encase::ShaderType)]
pub struct PreparedLight {
    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: render::Color,
    diffuse: render::Color,
    specular: render::Color,

    position: glam::Vec3,
}

impl Light {
    pub fn new(assets: &mut assets::Loader, scene_models: &mut scene::Meshes) -> Self {
        let transform = render::Transform::from_position(glam::vec3(0.0, 5.0, 0.0));

        let id = assets.models.load_tobj("assets/meshes/ico_sphere.obj");
        let model = assets.models.get_expect(id);
        let model_index = scene_models.upload_mesh(model.meshes[0].clone());

        let constant = 1.0;
        let linear = 0.07;
        let quadratic = 0.017;

        let ambient = render::Color::from_rgb_normalized(glam::vec3(1.0, 1.0, 1.0));
        let diffuse = render::Color::from_rgb_normalized(glam::vec3(1.0, 1.0, 1.0));
        let specular = render::Color::from_rgb_normalized(glam::vec3(1.0, 1.0, 1.0));

        Light {
            transform,
            mesh_index: model_index,

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
        let transform_index = resources.transforms.push(&transform) as u32;

        let instance =
            render::Instance::from_mesh_transform_indices(self.mesh_index, transform_index);
        let instance_index = resources.instances.push(instance) as u32;

        let color = self.diffuse;

        PreparedObject {
            instance_index,
            index_count: self.mesh_index.index_count as u32,
            index_offset: self.mesh_index.index_offset as u32,

            color,
        }
    }

    pub fn prepare_light(&self, resources: &mut scene::PrepareResources<'_>) {
        let prepared_light = PreparedLight {
            constant: self.constant,
            linear: self.linear,
            quadratic: self.quadratic,
            ambient: self.ambient,
            diffuse: self.diffuse,
            specular: self.specular,
            position: self.transform.position,
        };
        resources.lights.push(&prepared_light);
    }
}

impl PreparedObject {
    pub fn draw<'rpass>(
        self,
        _: &scene::RenderResources<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        render_pass.push_debug_group("wormhole light object draw");

        {
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::bytes_of(&self.color),
            );

            let index_start = self.index_offset / std::mem::size_of::<u32>() as u32;
            let index_end = index_start + self.index_count;

            let instance_start = self.instance_index;
            let instance_end = self.instance_index + 1;

            render_pass.draw_indexed(index_start..index_end, 0, instance_start..instance_end);
        }

        render_pass.pop_debug_group();
    }
}

impl render::traits::Bindable for PreparedLight {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("wormhole lights"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
                visibility: wgpu::ShaderStages::FRAGMENT,
            }],
        };

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout {
        &render_state.bind_groups.light
    }
}

impl render::traits::DynamicBufferWriteable for PreparedLight {}
