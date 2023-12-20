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
use crate::render;
use crate::systems;

use bevy_ecs::prelude::*;

mod meshes;
pub use meshes::{MeshIndex, Meshes};

pub struct Scene {
    world: World,
    schedule: Schedule,
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
    pub fn new(render_state: render::State, window: &winit::window::Window) -> Self {
        let mut world = World::new();
        world.insert_resource(input::State::new(window));
        world.insert_resource(physics::State::new());

        let mut assets = assets::Loader::new(&render_state);
        let mut meshes = Meshes::new(&render_state);

        let material_id = assets::MaterialId::from_path("cube_material");
        assets
            .materials
            .insert(material_id, render::Material::default());

        let model_id = assets
            .models
            .load_tobj("assets/meshes/cube.obj", material_id);
        let mesh = assets.models.get_expect(model_id).meshes[0].clone();
        world.spawn((
            components::Transform::new(),
            components::MeshRenderer::new(&mut meshes, mesh),
        ));
        world.spawn((
            components::Transform::from_position(glam::vec3(0.0, 5.0, 0.0)),
            components::Light::new(&mut assets, &mut meshes),
        ));

        world.insert_resource(assets);
        world.insert_resource(meshes);
        world.insert_resource(Buffers::new(&render_state));
        world.insert_resource(render::Camera::new(&render_state));
        world.insert_resource(render_state);

        let mut schedule = Schedule::default();
        schedule.add_systems(systems::render);
        schedule.add_systems(systems::input);

        Self { world, schedule }
    }

    pub fn process_event<T>(
        &mut self,
        event: &winit::event::Event<T>,
        target: &winit::event_loop::EventLoopWindowTarget<T>,
        window: &winit::window::Window,
    ) {
        let mut input_state = self.world.resource_mut::<input::State>();
        // let render_state = self.world.resource::<render::State>();

        if input_state.process(event, window) {
            if input_state.close_requested()
                || input_state
                    .keyboard
                    .pressed(winit::keyboard::KeyCode::Escape)
            {
                target.exit();
            }

            self.schedule.run(&mut self.world);
        }
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
