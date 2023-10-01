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
use crate::input;
use crate::render;

use crate::light;
use crate::object;

use itertools::Itertools;
use std::time::Instant;

mod models;
pub use models::{ModelIndex, Models};

pub struct Scene {
    objects: Vec<object::Object>,
    lights: Vec<light::Light>,

    camera: render::Camera,
    buffers: Buffers,
    models: Models,

    last_update: Instant,
}

pub struct Buffers {
    transforms: render::buffer::dynamic::Buffer<render::Transform>,
    lights: render::buffer::dynamic::Buffer<light::PreparedLight>,
    camera: render::buffer::single::Buffer<render::Camera>,

    gbuffer: render::buffer::geometry::Buffer,
    screen_vertices: wgpu::Buffer,
}

impl Buffers {
    pub fn new(render_state: &render::State) -> Self {
        let transforms =
            render::buffer::dynamic::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let lights =
            render::buffer::dynamic::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let camera = render::buffer::single::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let gbuffer = render::buffer::geometry::Buffer::new(render_state);

        let screen_vertices = Scene::create_screen_vertex_buffer(render_state);

        Self {
            transforms,
            lights,
            camera,
            gbuffer,
            screen_vertices,
        }
    }
}

pub struct PrepareResources<'buf> {
    pub transforms: render::buffer::dynamic::Writer<'buf, render::Transform>,
    pub lights: render::buffer::dynamic::Writer<'buf, light::PreparedLight>,
    pub camera: render::buffer::single::Writer<'buf, render::Camera>,
}

pub struct RenderResources<'res> {
    pub transform: &'res wgpu::BindGroup,
    pub lights: &'res wgpu::BindGroup,
    pub camera: &'res wgpu::BindGroup,

    pub vertex_buffer: &'res wgpu::Buffer,
    pub index_buffer: &'res wgpu::Buffer,
}

impl Scene {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader) -> Self {
        let camera = render::Camera::new(render_state);

        let buffers = Buffers::new(render_state);
        let mut models = Models::new(render_state);

        let objects = vec![object::Object::new(render_state, assets, &mut models)];
        let lights = vec![light::Light::new(assets, &mut models)];

        let last_update = Instant::now();

        Self {
            objects,
            lights,

            camera,
            buffers,
            models,

            last_update,
        }
    }

    pub fn update(&mut self, render_state: &render::State, input_state: &input::State) {
        let before = std::mem::replace(&mut self.last_update, Instant::now());
        let dt = (self.last_update - before).as_secs_f32();

        if input_state.new_window_size().is_some() {
            self.buffers.gbuffer.resize_to_screen(render_state);
        }

        for object in self.objects.iter_mut() {
            object.update(dt)
        }

        self.camera.update(input_state, dt);
    }

    // Currently rendering has a prepare->finish->deferred draw->lighting design.
    // Every frame, scene resources are prepared (written to CPU-side buffers) that are then uploaded to the GPU (finish).
    //
    // These resources are used for a basic geometry pass, and then a super basic lighting pass is performed.
    pub fn render(&mut self, render_state: &render::State) {
        let mut encoder =
            render_state
                .wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render pass encoder"),
                });

        // Prepare everything for rendering
        encoder.push_debug_group("Scene prep");

        self.models.write_unwritten(render_state, &mut encoder);

        let (vertex_buffer, index_buffer) = self.models.as_buffers();

        let mut resources = PrepareResources {
            transforms: self.buffers.transforms.start_write(),
            lights: self.buffers.lights.start_write(),
            camera: self.buffers.camera.start_write(),
        };

        let prepared_objects = self
            .objects
            .iter()
            .map(|o| o.prepare(&mut resources))
            .collect_vec();

        self.lights
            .iter()
            .for_each(|l| l.prepare_light(&mut resources));

        let prepared_light_objects = self
            .lights
            .iter()
            .map(|l| l.prepare_object(&mut resources))
            .collect_vec();

        resources
            .camera
            .write(&self.camera)
            .expect("failed to write camera data");

        encoder.pop_debug_group();

        encoder.push_debug_group("wormhole deferred render pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole deferred render pass"),
            color_attachments: &self.buffers.gbuffer.as_color_attachments(),
            depth_stencil_attachment: Some(self.buffers.gbuffer.depth_stencil_attachment_initial()),
        });

        let render_resources = RenderResources {
            transform: resources.transforms.finish(render_state),
            lights: resources.lights.finish(render_state),
            camera: resources.camera.finish(render_state),

            vertex_buffer,
            index_buffer,
        };

        render_pass.set_pipeline(&render_state.pipelines.object);
        render_pass.set_bind_group(0, render_resources.camera, &[]);
        render_pass.set_bind_group(1, render_resources.transform, &[]);
        for prepared in prepared_objects {
            prepared.draw(&render_resources, &mut render_pass);
        }

        drop(render_pass);

        encoder.pop_debug_group();

        encoder.push_debug_group("wormhole lighting pass");

        let output = match render_state.wgpu.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                render_state.reconfigure_surface();

                return;
            }
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of gpu memory. exiting"),
        };

        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole lighting pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_state.pipelines.light);

        render_pass.set_vertex_buffer(0, self.buffers.screen_vertices.slice(..));

        render_pass.set_bind_group(0, render_resources.camera, &[]);
        render_pass.set_bind_group(1, render_resources.lights, &[]);
        render_pass.set_bind_group(2, &self.buffers.gbuffer.bind_group, &[]);

        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(&(self.lights.len() as u32)),
        );

        render_pass.draw(0..6, 0..1);

        drop(render_pass);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole light box pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(self.buffers.gbuffer.depth_stencil_attachment()),
        });

        render_pass.set_pipeline(&render_state.pipelines.light_object);
        render_pass.set_bind_group(0, render_resources.camera, &[]);
        render_pass.set_bind_group(1, render_resources.transform, &[]);
        for light in prepared_light_objects {
            light.draw(&render_resources, &mut render_pass);
        }

        drop(render_pass);

        encoder.pop_debug_group();

        render_state
            .wgpu
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();
    }

    fn create_screen_vertex_buffer(render_state: &render::State) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let screen_mesh = &[
            // 2
            render::Vertex {
                position: glam::vec3(1.0, -1.0, 0.0),
                tex_coords: glam::vec2(1.0, 1.0),
                ..Default::default()
            },
            // 1
            render::Vertex {
                position: glam::vec3(1.0, 1.0, 0.0),
                tex_coords: glam::vec2(1.0, 0.0),
                ..Default::default()
            },
            // 0
            render::Vertex {
                position: glam::vec3(-1.0, 1.0, 0.0),
                tex_coords: glam::vec2(0.0, 0.0),
                ..Default::default()
            },
            // 2
            render::Vertex {
                position: glam::vec3(1.0, -1.0, 0.0),
                tex_coords: glam::vec2(1.0, 1.0),
                ..Default::default()
            },
            // 0
            render::Vertex {
                position: glam::vec3(-1.0, 1.0, 0.0),
                tex_coords: glam::vec2(0.0, 0.0),
                ..Default::default()
            },
            // 3
            render::Vertex {
                position: glam::vec3(-1.0, -1.0, 0.0),
                tex_coords: glam::vec2(0.0, 1.0),
                ..Default::default()
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
}
