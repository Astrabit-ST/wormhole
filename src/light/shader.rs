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

impl super::Light {
    pub fn create_light_object_render_pipeline(
        render_state: &render::state::BindGroupsCreated,
    ) -> wgpu::RenderPipeline {
        let layout =
            render_state
                .wgpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("object render pipeline layout"),
                    bind_group_layouts: &[
                        &render_state.bind_groups.camera,
                        &render_state.bind_groups.transform,
                    ],
                    push_constant_ranges: &[
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: 0..4,
                        },
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::FRAGMENT,
                            range: 12..32,
                        },
                    ],
                });

        let shader = render_state
            .wgpu
            .device
            .create_shader_module(wgpu::include_wgsl!("light_object.wgsl"));

        render_state
            .wgpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("light object render pipeline"),
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
                        format: render_state.wgpu.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    pub fn create_light_render_pipeline(
        render_state: &render::state::BindGroupsCreated,
    ) -> wgpu::RenderPipeline {
        let layout =
            render_state
                .wgpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("lighting render pipeline layout"),
                    bind_group_layouts: &[
                        &render_state.bind_groups.camera,
                        &render_state.bind_groups.transform,
                        &render_state.bind_groups.gbuffer,
                    ],
                    push_constant_ranges: &[
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: 0..4,
                        },
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::FRAGMENT,
                            range: 8..48,
                        },
                    ],
                });

        let shader = render_state
            .wgpu
            .device
            .create_shader_module(wgpu::include_wgsl!("light.wgsl"));

        render_state
            .wgpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("light render pipeline"),
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
                        format: render_state.wgpu.surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
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
            })
    }
}
