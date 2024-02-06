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
use crate::components;
use crate::input;
use crate::physics;
use crate::player;
use crate::render;
use crate::time;

use bevy_ecs::prelude::*;
use rapier3d::prelude::*;

use bevy_ecs::schedule::ScheduleLabel;

mod schedules;
use bevy_ecs::system::SystemState;
pub use schedules::*;

mod world_builder;
pub use world_builder::WorldBuilder;

mod meshes;
pub use meshes::{MeshIndex, Meshes};

pub struct Scene {
    pub world: World,
}

#[derive(Resource)]
pub struct Buffers {
    pub transforms: render::buffer::dynamic::Buffer<components::Transform>,
    pub lights: render::buffer::dynamic::Buffer<components::light::PreparedLight>,

    pub instances: render::buffer::instances::Buffer,

    pub gbuffer: render::buffer::geometry::Buffer,
    pub screen_vertices: wgpu::Buffer,
}

impl Buffers {
    pub fn new(render_state: &render::State) -> Self {
        let transforms =
            render::buffer::dynamic::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let lights =
            render::buffer::dynamic::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let instances =
            render::buffer::instances::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let gbuffer = render::buffer::geometry::Buffer::new(render_state);

        let screen_vertices = create_screen_vertex_buffer(render_state);

        Self {
            transforms,
            lights,
            instances,
            gbuffer,
            screen_vertices,
        }
    }
}

pub struct PrepareResources<'buf> {
    pub transforms: render::buffer::dynamic::Writer<'buf, components::Transform>,
    pub lights: render::buffer::dynamic::Writer<'buf, components::light::PreparedLight>,
    pub instances: render::buffer::instances::Writer<'buf>,
    pub assets: &'buf assets::Loader,
}

impl Scene {
    pub fn new(render_state: render::State) -> Self {
        let mut builder = WorldBuilder::new();

        builder
            .insert_resource(Meshes::new(&render_state))
            .insert_resource(Buffers::new(&render_state));

        schedules::init_into(&mut builder);
        time::init_into(&mut builder);
        physics::init_into(&mut builder);
        input::init_into(&mut builder);
        assets::init_into(&render_state, &mut builder);
        player::init_into(&render_state, &mut builder);
        render::init_into(render_state, &mut builder);

        let mut world = builder.build();

        let mut system_state = SystemState::<(
            ResMut<assets::Loader>,
            Res<render::State>,
            ResMut<Meshes>,
            ResMut<physics::State>,
            Commands,
        )>::from_world(&mut world);

        let (mut assets, render_state, mut meshes, mut physics_state, mut commands) =
            system_state.get_mut(&mut world);
        let physics_state = &mut *physics_state;

        let diffuse_id = assets
            .textures
            .load_from_path(&render_state, "assets/textures/cube-diffuse.jpg");
        let normal_id = assets
            .textures
            .load_from_path(&render_state, "assets/textures/cube-normal.png");

        let material_id = assets::MaterialId::from_path("cube_material");
        assets.materials.insert(
            material_id,
            render::Material {
                base_color_texture: Some(diffuse_id),
                normal_texture: Some(normal_id),
                ..Default::default()
            },
        );

        let model_id = assets
            .models
            .load_tobj("assets/meshes/cube.obj", material_id);
        let model = assets.models.get_expect(model_id);
        let mesh = model.meshes[0].clone();

        let light = components::Light::new(&mut assets, &mut meshes);
        commands.spawn((
            components::Transform::from_position(glam::vec3(0.0, 5.0, 0.0)),
            light,
        ));

        let mesh_renderer = components::MeshRenderer::new(&mut meshes, mesh.clone());
        let mut entity_builder = commands.spawn((components::Transform::default(), mesh_renderer));

        let rigid_body = RigidBodyBuilder::dynamic().additional_mass(1.0).build();
        let rigid_body = physics::RigidBody::new(physics_state, entity_builder.id(), rigid_body);

        let collider = ColliderBuilder::cuboid(1.1, 1.1, 1.1)
            .restitution(0.9)
            .build();
        physics_state.collider_set.insert_with_parent(
            collider,
            rigid_body.handle,
            &mut physics_state.rigid_body_set,
        );
        entity_builder.insert(rigid_body);

        let mesh_renderer = components::MeshRenderer::new(&mut meshes, mesh);
        commands.spawn((
            components::Transform::from_position_scale(
                glam::vec3(0.0, -4.0, 0.0),
                glam::vec3(100.0, 0.1, 100.0),
            ),
            mesh_renderer,
        ));

        let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0)
            .translation(vector![0.0, -4.0, 0.0])
            .build();
        physics_state.collider_set.insert(collider);

        system_state.apply(&mut world);

        Self { world }
    }

    pub fn update(&mut self) {
        self.world.run_schedule(Main.intern());
        self.world.clear_trackers();
    }
}

fn create_screen_vertex_buffer(render_state: &render::State) -> wgpu::Buffer {
    use wgpu::util::DeviceExt;

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: glam::Vec3,
        tex_coords: glam::Vec2,
    }

    let screen_mesh = &[
        // 2
        Vertex {
            position: glam::vec3(1.0, -1.0, 0.0),
            tex_coords: glam::vec2(1.0, 1.0),
        },
        // 1
        Vertex {
            position: glam::vec3(1.0, 1.0, 0.0),
            tex_coords: glam::vec2(1.0, 0.0),
        },
        // 0
        Vertex {
            position: glam::vec3(-1.0, 1.0, 0.0),
            tex_coords: glam::vec2(0.0, 0.0),
        },
        // 2
        Vertex {
            position: glam::vec3(1.0, -1.0, 0.0),
            tex_coords: glam::vec2(1.0, 1.0),
        },
        // 0
        Vertex {
            position: glam::vec3(-1.0, 1.0, 0.0),
            tex_coords: glam::vec2(0.0, 0.0),
        },
        // 3
        Vertex {
            position: glam::vec3(-1.0, -1.0, 0.0),
            tex_coords: glam::vec2(0.0, 1.0),
        },
    ];

    render_state
        .wgpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("wormhole screen vertex buffer"),
            contents: bytemuck::cast_slice(screen_mesh),
            usage: wgpu::BufferUsages::VERTEX,
        })
}
