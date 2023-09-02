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
    depth_texture: render::DepthTexture,

    last_update: Instant,
}

pub struct Buffers {
    pub transform: render::dynamic::Buffer<render::Transform>,
    pub camera: render::single::Buffer<render::Camera>,
}

pub struct PrepareResources<'buf> {
    pub transform: render::dynamic::Writer<'buf, render::Transform>,
    pub camera: render::single::Writer<'buf, render::Camera>,
}

pub struct RenderResources<'res> {
    pub transform: &'res wgpu::BindGroup,
    pub camera: &'res wgpu::BindGroup,
}

impl Scene {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader) -> Self {
        let camera = render::Camera::new(render_state);
        let objects = (0..100)
            .map(|i| object::Object::new(render_state, assets, i))
            .collect_vec();

        let transform_buffer = render::dynamic::Buffer::new(
            render_state,
            wgpu::BufferUsages::empty(),
            render::Transform::bind_group_layout(),
        );

        let camera_buffer = render::single::Buffer::new(
            render_state,
            wgpu::BufferUsages::empty(),
            render::Camera::bind_group_layout(),
        );

        let buffers = Buffers {
            transform: transform_buffer,
            camera: camera_buffer,
        };

        let depth_texture = render::DepthTexture::builder().build(render_state);

        let last_update = Instant::now();

        Self {
            camera,
            objects,

            buffers,
            depth_texture,

            last_update,
        }
    }

    pub fn update(&mut self, render_state: &render::State, input_state: &input::State) {
        let before = std::mem::replace(&mut self.last_update, Instant::now());
        let dt = (self.last_update - before).as_secs_f32();

        if input_state.new_window_size().is_some() {
            self.depth_texture.resize(render_state);
        }

        for object in self.objects.iter_mut() {
            object.update(dt)
        }

        self.camera.update(input_state, dt);
    }

    pub fn render(&mut self, render_state: &render::State) {
        let output = match render_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                render_state.reconfigure_surface();

                return;
            }
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of gpu memory. exiting"),
        };

        let mut encoder =
            render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render pass encoder"),
                });

        // Prepare everything for rendering
        encoder.push_debug_group("Scene prep");

        let mut resources = PrepareResources {
            transform: self.buffers.transform.start_write(),
            camera: self.buffers.camera.start_write(),
        };

        let prepared_objects = self
            .objects
            .iter()
            .map(|o| o.prepare(&mut resources))
            .collect_vec();

        resources
            .camera
            .write(&self.camera)
            .expect("failed to write camera data");

        encoder.pop_debug_group();

        // Do rendering
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let render_resources = RenderResources {
            transform: resources.transform.finish(render_state),
            camera: resources.camera.finish(render_state),
        };

        for prepared in prepared_objects {
            prepared.draw(&render_resources, &mut render_pass);
        }

        drop(render_pass);

        render_state.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }
}
