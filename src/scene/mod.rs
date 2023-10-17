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

use itertools::Itertools;
use std::time::Instant;

mod meshes;
pub use meshes::{MeshIndex, Meshes};

pub struct Scene {
    objects: Vec<render::Object>,
    lights: Vec<render::Light>,

    camera: render::Camera,
    buffers: Buffers,
    meshes: Meshes,

    last_update: Instant,
}

pub struct Buffers {
    transforms: render::buffer::dynamic::Buffer<render::Transform>,
    lights: render::buffer::dynamic::Buffer<render::light::PreparedLight>,

    instances: render::buffer::instances::Buffer,

    gbuffer: render::buffer::geometry::Buffer,
    screen_vertices: wgpu::Buffer,
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

        let screen_vertices = Scene::create_screen_vertex_buffer(render_state);

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
    pub transforms: render::buffer::dynamic::Writer<'buf, render::Transform>,
    pub lights: render::buffer::dynamic::Writer<'buf, render::light::PreparedLight>,
    pub instances: render::buffer::instances::Writer<'buf>,
}

impl Scene {
    pub fn new(render_state: &render::State, assets: &mut assets::Loader) -> Self {
        let camera = render::Camera::new(render_state);

        let mut meshes = Meshes::new(render_state);

        let mut objects = vec![];

        for path in ["assets/WaterBottle/glTF/WaterBottle.gltf"] {
            Self::load_gltf(render_state, path, &mut meshes, assets, &mut objects)
        }

        let lights = vec![render::Light::new(assets, &mut meshes)];

        let buffers = Buffers::new(render_state);

        let last_update = Instant::now();

        Self {
            objects,
            lights,

            camera,
            buffers,
            meshes,

            last_update,
        }
    }

    fn load_gltf(
        render_state: &render::State,
        path: impl AsRef<camino::Utf8Path>,
        meshes: &mut Meshes,
        assets: &mut assets::Loader,
        objects: &mut Vec<render::Object>,
    ) {
        let path = path.as_ref();
        assets.load_gltf(render_state, path);

        let gltf_id = assets::GltfId::from_path(path);
        let gltf = assets.gltf.get_expect(gltf_id);

        fn process_node(
            gltf_id: assets::GltfId,
            node: gltf::Node<'_>,
            meshes: &mut Meshes,
            assets: &assets::Loader,
            objects: &mut Vec<render::Object>,
        ) {
            let transform = node.transform().into();
            if let Some(mesh) = node.mesh() {
                let model = assets
                    .models
                    .get_expect(assets::ModelId::Gltf(gltf_id, mesh.index()));
                let object = render::Object::new(meshes, transform, model);
                objects.push(object);
            }
            for node in node.children() {
                process_node(gltf_id, node, meshes, assets, objects)
            }
        }
        for node in gltf.document.nodes() {
            process_node(gltf_id, node, meshes, assets, objects);
        }
    }

    pub fn update(&mut self, render_state: &render::State, input_state: &input::State) {
        let before = std::mem::replace(&mut self.last_update, Instant::now());
        let dt = (self.last_update - before).as_secs_f32();

        if input_state.new_window_size().is_some() {
            self.buffers.gbuffer.resize_to_screen(render_state);
        }

        self.camera.update(input_state, dt);
    }

    // Currently rendering has a prepare->finish->deferred draw->lighting design.
    // Every frame, scene resources are prepared (written to CPU-side buffers) that are then uploaded to the GPU (finish).
    //
    // These resources are used for a basic geometry pass, and then a super basic lighting pass is performed.
    pub fn render(&mut self, render_state: &render::State, assets: &mut assets::Loader) {
        let mut encoder =
            render_state
                .wgpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render pass encoder"),
                });

        // Prepare everything for rendering
        encoder.push_debug_group("Scene prep");

        self.meshes.write_unwritten(render_state, &mut encoder);

        let (vertex_buffers, index_buffer) = self.meshes.as_bind_group_index_buffer();

        let mut resources = PrepareResources {
            transforms: self.buffers.transforms.start_write(),
            lights: self.buffers.lights.start_write(),
            instances: self.buffers.instances.start_write(),
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

        let instance_buffer = resources.instances.finish(render_state);

        let light_buffer = resources.lights.finish(render_state);
        let light_data = render::BindGroupBuilder::new()
            .append_buffer(light_buffer)
            .build(
                &render_state.wgpu.device,
                Some("wormhole light data"),
                &render_state.bind_groups.light_data,
            );

        let transform_buffer = resources.transforms.finish(render_state);
        let object_data = render::BindGroupBuilder::new()
            .append_buffer(transform_buffer)
            .append_buffer(vertex_buffers[0])
            .append_buffer(vertex_buffers[1])
            .append_buffer(vertex_buffers[2])
            .append_buffer(vertex_buffers[3])
            .append_buffer(vertex_buffers[4])
            .build(
                &render_state.wgpu.device,
                Some("wormhole object data"),
                &render_state.bind_groups.object_data,
            );

        let default_texture_sampler = &self.buffers.gbuffer.sampler;
        let material_buffer = assets
            .materials
            .get_or_update_buffer(render_state, &assets.textures);
        let texture_views = assets.textures.get_texture_views();
        let material_data = render::BindGroupBuilder::new()
            .append_sampler(default_texture_sampler)
            .append_texture_view_array(&texture_views)
            .append_buffer(material_buffer)
            .build(
                &render_state.wgpu.device,
                Some("wormhole material data"),
                &render_state.bind_groups.materials,
            );

        encoder.pop_debug_group();

        encoder.push_debug_group("wormhole deferred render pass");

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("wormhole deferred render pass"),
            color_attachments: &self.buffers.gbuffer.as_color_attachments(),
            depth_stencil_attachment: Some(self.buffers.gbuffer.depth_stencil_attachment_initial()),
        });

        render_pass.set_pipeline(&render_state.pipelines.object);

        render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(0, &object_data, &[]);
        render_pass.set_bind_group(1, &material_data, &[]);

        for prepared in prepared_objects {
            prepared.draw(&mut render_pass);
        }

        drop(render_pass);

        encoder.pop_debug_group();

        encoder.push_debug_group("wormhole lighting pass");

        let output = match render_state.wgpu.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(error @ (wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost)) => {
                render_state.reconfigure_surface();
                eprintln!("surface error: {error:#?}");

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

        render_pass.set_bind_group(0, &light_data, &[]);
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

        render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.set_bind_group(0, &object_data, &[]);

        for light in prepared_light_objects {
            light.draw(&mut render_pass);
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
}
