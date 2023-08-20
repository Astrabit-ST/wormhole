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
use crate::render;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct Shader {
    pipeline: wgpu::RenderPipeline,
}

static SHADER: OnceCell<Shader> = OnceCell::new();

impl Shader {
    pub fn create(render_state: &render::State) {
        let layout = render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("object render pipeline layout"),
                bind_group_layouts: &[
                    render::Camera::bind_group_layout(),
                    render::Transform::bind_group_layout(),
                    super::Textures::bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let shader = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("object.wgsl"));

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("object render pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[render::Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: render_state.surface_config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        SHADER
            .set(Self { pipeline })
            .expect("shader already initialized");
    }

    pub fn bind(render_pass: &mut wgpu::RenderPass<'_>) {
        let shader = SHADER.get().expect("shader not initialized");
        render_pass.set_pipeline(&shader.pipeline);
    }
}
