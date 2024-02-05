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
use crate::player;
use crate::render;
use crate::scene;

use bevy_ecs::prelude::*;

use itertools::Itertools;

pub fn render(
    render_state: Res<render::State>,
    mut buffers: ResMut<scene::Buffers>,
    mut meshes: ResMut<scene::Meshes>,
    mut assets: ResMut<assets::Loader>,
    player: Res<player::Player>,
    object_query: Query<(&components::Transform, &components::MeshRenderer)>,
    light_query: Query<(&components::Transform, &components::Light)>,
) {
    let mut encoder =
        render_state
            .wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render pass encoder"),
            });

    let assets = &mut *assets;
    let buffers = &mut *buffers;

    // Prepare everything for rendering
    encoder.push_debug_group("Scene prep");

    meshes.write_unwritten(&render_state, &mut encoder);

    let (vertex_buffers, index_buffer) = meshes.as_bind_group_index_buffer();

    let default_texture_sampler = &buffers.gbuffer.sampler;
    let material_buffer = assets
        .materials
        .get_or_update_buffer(&render_state, &assets.textures);
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

    let mut resources = scene::PrepareResources {
        transforms: buffers.transforms.start_write(),
        lights: buffers.lights.start_write(),
        instances: buffers.instances.start_write(),
        assets,
    };

    let prepared_objects = object_query
        .iter()
        .map(|(transform, object)| {
            let transform_index = resources.transforms.push(transform) as u32;
            object.prepare(transform_index, &mut resources)
        })
        .collect_vec();

    let prepared_light_objects = light_query
        .iter()
        .map(|(transform, light)| {
            let transform_index = resources.transforms.push(transform) as u32;
            light.prepare_light(transform.position, &mut resources);
            light.prepare_object(transform_index, &mut resources)
        })
        .collect_vec();

    let instance_buffer = resources.instances.finish(&render_state);

    let light_buffer = resources.lights.finish(&render_state);
    let light_data = render::BindGroupBuilder::new()
        .append_buffer(light_buffer)
        .build(
            &render_state.wgpu.device,
            Some("wormhole light data"),
            &render_state.bind_groups.light_data,
        );

    let transform_buffer = resources.transforms.finish(&render_state);
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

    let camera_data = player.camera.as_camera_data(player.transform);

    encoder.pop_debug_group();

    encoder.push_debug_group("wormhole deferred render pass");

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("wormhole deferred render pass"),
        color_attachments: &buffers.gbuffer.as_color_attachments(),
        depth_stencil_attachment: Some(buffers.gbuffer.depth_stencil_attachment_initial()),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    render_pass.set_pipeline(&render_state.pipelines.object);

    render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    render_pass.set_bind_group(0, &object_data, &[]);
    render_pass.set_bind_group(1, &material_data, &[]);

    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::bytes_of(&camera_data.view_proj),
    );

    for prepared in prepared_objects {
        prepared.draw(&mut render_pass);
    }

    drop(render_pass);

    encoder.pop_debug_group();

    encoder.push_debug_group("wormhole lighting pass");

    let output = match render_state.wgpu.surface.get_current_texture() {
        Ok(texture) => texture,
        Err(error @ (wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost)) => {
            render_state
                .wgpu
                .surface
                .configure(&render_state.wgpu.device, &render_state.wgpu.surface_config);

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
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    render_pass.set_pipeline(&render_state.pipelines.light);

    render_pass.set_vertex_buffer(0, buffers.screen_vertices.slice(..));

    render_pass.set_bind_group(0, &light_data, &[]);
    render_pass.set_bind_group(1, &buffers.gbuffer.bind_group, &[]);

    // FIXME: clunky
    #[repr(C)]
    #[derive(Clone, Copy)]
    #[derive(bytemuck::Pod, bytemuck::Zeroable)]
    struct LightPushConstants {
        light_count: u32,
        view_pos: glam::Vec3,
    }
    render_pass.set_push_constants(
        wgpu::ShaderStages::FRAGMENT,
        0,
        bytemuck::bytes_of(&LightPushConstants {
            light_count: prepared_light_objects.len() as u32,
            view_pos: camera_data.view_pos,
        }),
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
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(buffers.gbuffer.depth_stencil_attachment()),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    render_pass.set_pipeline(&render_state.pipelines.light_object);

    render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    render_pass.set_bind_group(0, &object_data, &[]);

    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::bytes_of(&camera_data.view_proj),
    );

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
