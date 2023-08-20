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
use crate::input;
use crate::render;

use crate::object;

use std::time::Instant;

pub struct Scene {
    camera: render::Camera,
    objects: Vec<object::Object>,

    last_update: Instant,
}

impl Scene {
    pub fn new(render_state: &render::State) -> Self {
        let camera = render::Camera::new(render_state);
        let objects = vec![object::Object::new(render_state)];

        let last_update = Instant::now();

        Self {
            camera,
            objects,
            last_update,
        }
    }

    pub fn update(&mut self, render_state: &render::State, input: &input::State) {
        let before = std::mem::replace(&mut self.last_update, Instant::now());
        let dt = (self.last_update - before).as_secs_f32();

        self.camera.update(render_state, input, dt);
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
            depth_stencil_attachment: None,
        });

        object::Shader::bind(&mut render_pass);
        self.camera.bind(&mut render_pass, 0);
        for object in self.objects.iter() {
            object.draw(&mut render_pass);
        }

        drop(render_pass);

        render_state.queue.submit(std::iter::once(encoder.finish()));

        output.present();
    }
}
