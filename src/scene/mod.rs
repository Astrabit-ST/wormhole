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

use crate::object;

use itertools::Itertools;
use std::time::Instant;

pub struct Scene {
    camera: render::Camera,
    objects: Vec<object::Object>,

    buffers: Buffers,

    last_update: Instant,
}

pub struct Buffers {
    transform: render::buffer::dynamic::Buffer<render::Transform>,
    camera: render::buffer::single::Buffer<render::Camera>,
    mesh: render::buffer::mesh::Buffer,

    geometry: render::buffer::geometry::Buffer,

    // Temporary hack before I add in a decent light system.
    // Just for testing
    light_temporary_hack: render::Mesh,
}

pub struct PrepareResources<'buf> {
    pub transform: render::buffer::dynamic::Writer<'buf, render::Transform>,
    pub camera: render::buffer::single::Writer<'buf, render::Camera>,
    pub mesh: render::buffer::mesh::Writer<'buf>,
}

pub struct RenderResources<'res> {
    pub transform: &'res wgpu::BindGroup,
    pub camera: &'res wgpu::BindGroup,

    pub vertices: &'res wgpu::Buffer,
    pub indices: &'res wgpu::Buffer,
}

impl Scene {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader) -> Self {
        let camera = render::Camera::new(render_state);
        let objects = (0..400)
            .map(|i| object::Object::new(render_state, assets, i))
            .collect_vec();

        let transform_buffer =
            render::buffer::dynamic::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let camera_buffer =
            render::buffer::single::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let mesh_buffer =
            render::buffer::mesh::Buffer::new(render_state, wgpu::BufferUsages::empty());

        let gbuffer = render::buffer::geometry::Buffer::new(render_state);

        let light_temporary_hack = render::Mesh::new(
            &[
                render::Vertex {
                    position: glam::vec3(-1.0, 1.0, 0.0),
                    tex_coords: glam::vec2(0.0, 0.0),
                    ..Default::default()
                },
                render::Vertex {
                    position: glam::vec3(1.0, 1.0, 0.0),
                    tex_coords: glam::vec2(1.0, 0.0),
                    ..Default::default()
                },
                render::Vertex {
                    position: glam::vec3(1.0, -1.0, 0.0),
                    tex_coords: glam::vec2(1.0, 1.0),
                    ..Default::default()
                },
                render::Vertex {
                    position: glam::vec3(-1.0, -1.0, 0.0),
                    tex_coords: glam::vec2(0.0, 1.0),
                    ..Default::default()
                },
            ],
            &[
                2, 1, 0, //
                2, 0, 3,
            ],
        );

        let buffers = Buffers {
            transform: transform_buffer,
            camera: camera_buffer,
            mesh: mesh_buffer,

            geometry: gbuffer,

            light_temporary_hack,
        };

        let last_update = Instant::now();

        Self {
            camera,
            objects,

            buffers,

            last_update,
        }
    }

    pub fn update(&mut self, render_state: &render::State, input_state: &input::State) {
        let before = std::mem::replace(&mut self.last_update, Instant::now());
        let dt = (self.last_update - before).as_secs_f32();

        if input_state.new_window_size().is_some() {
            self.buffers.geometry.resize_to_screen(render_state);
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

        let mut resources = PrepareResources {
            transform: self.buffers.transform.start_write(),
            camera: self.buffers.camera.start_write(),
            mesh: self.buffers.mesh.start_write(),
        };

        let prepared_objects = self
            .objects
            .iter()
            .map(|o| o.prepare(&mut resources))
            .collect_vec();
        let prepared_light_hack = self.buffers.light_temporary_hack.prepare(&mut resources);

        resources
            .camera
            .write(&self.camera)
            .expect("failed to write camera data");

        encoder.pop_debug_group();

        encoder.push_debug_group("wormhole deferred render pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole deferred render pass"),
            color_attachments: &self.buffers.geometry.as_color_attachments(),
            depth_stencil_attachment: self.buffers.geometry.depth_stencil_attachment(),
        });

        let (vertex_buffer, index_buffer) = resources.mesh.finish(render_state);

        let render_resources = RenderResources {
            transform: resources.transform.finish(render_state),
            camera: resources.camera.finish(render_state),

            vertices: vertex_buffer,
            indices: index_buffer,
        };

        render_pass.set_pipeline(&render_state.pipelines.object);
        render_pass.set_bind_group(0, render_resources.camera, &[]);
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
        render_pass.set_bind_group(0, render_resources.camera, &[]);
        render_pass.set_bind_group(1, &self.buffers.geometry.bind_group, &[]);
        prepared_light_hack.draw(&render_resources, &mut render_pass);

        drop(render_pass);

        encoder.pop_debug_group();

        render_state
            .wgpu
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();

        std::process::exit(1)
    }
}
