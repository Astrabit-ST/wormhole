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
use crate::systems;
use crate::time;

use bevy_ecs::event::event_queue_update_system;
use bevy_ecs::prelude::*;

mod schedules;
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::schedule::ScheduleLabel;
pub use schedules::*;

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

struct WorldBuilder {
    world: World,
}

impl WorldBuilder {
    fn new() -> Self {
        let mut world = World::new();
        world.init_resource::<Schedules>();

        Self { world }
    }

    fn add_schedule(mut self, schedule: Schedule) -> Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.insert(schedule);

        self
    }

    fn init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.world.init_resource::<R>();
        self
    }

    fn insert_resource(mut self, resource: impl Resource) -> Self {
        self.world.insert_resource(resource);
        self
    }

    fn add_systems<M>(
        mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> Self {
        let schedule = schedule.intern();
        let mut schedules = self.world.resource_mut::<Schedules>();

        if let Some(schedule) = schedules.get_mut(schedule) {
            schedule.add_systems(systems);
        } else {
            let mut new_schedule = Schedule::new(schedule);
            new_schedule.add_systems(systems);
            schedules.insert(new_schedule);
        }

        self
    }

    fn add_event<T: Event>(self) -> Self {
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>().add_systems(
                First,
                bevy_ecs::event::event_update_system::<T>
                    .run_if(bevy_ecs::event::event_update_condition::<T>),
            )
        } else {
            self
        }
    }

    fn build(self) -> World {
        self.world
    }
}

impl Scene {
    pub fn new(render_state: render::State) -> Self {
        let mut main_schedule = Schedule::new(Main);
        main_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        let mut fixed_main_shedule = Schedule::new(FixedMain);
        fixed_main_shedule.set_executor_kind(ExecutorKind::SingleThreaded);

        let mut fixed_main_loop_schedule = Schedule::new(RunFixedMainLoop);
        fixed_main_loop_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        let world = WorldBuilder::new()
            .add_schedule(main_schedule)
            .add_schedule(fixed_main_shedule)
            .add_schedule(fixed_main_loop_schedule)
            .init_resource::<MainScheduleOrder>()
            .init_resource::<FixedMainScheduleOrder>()
            .init_resource::<time::Time>()
            .init_resource::<time::Time<time::Real>>()
            .init_resource::<time::Time<time::Virtual>>()
            .init_resource::<time::Time<time::Fixed>>()
            .init_resource::<bevy_ecs::event::EventUpdateSignal>()
            .insert_resource(physics::State::new())
            .insert_resource(input::State::new())
            .insert_resource(player::Player::new(&render_state))
            .insert_resource(assets::Loader::new(&render_state))
            .insert_resource(Meshes::new(&render_state))
            .insert_resource(Buffers::new(&render_state))
            .insert_resource(render_state)
            .add_event::<input::KeyboardEvent>()
            .add_event::<input::KeyboardEvent>()
            .add_event::<input::MouseButtonEvent>()
            .add_event::<input::MouseWheel>()
            .add_event::<input::MouseMotion>()
            .add_event::<input::WindowResized>()
            .add_event::<input::CloseRequested>()
            .add_event::<input::Exit>()
            .add_systems(Main, Main::run_main)
            .add_systems(FixedMain, FixedMain::run_fixed_main)
            .add_systems(
                First,
                (
                    systems::update_time,
                    systems::update_virtual_time.after(systems::update_time),
                )
                    .in_set(systems::TimeSystem),
            )
            .add_systems(
                PreUpdate,
                (systems::keyboard_input_system.in_set(systems::InputSystem),),
            )
            .add_systems(RunFixedMainLoop, systems::run_fixed_main_schedule)
            .add_systems(FixedUpdate, systems::input)
            .add_systems(FixedPostUpdate, event_queue_update_system)
            .add_systems(Update, systems::render)
            .build();

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
