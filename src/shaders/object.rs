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

pub fn create_render_pipeline(
    composer: &mut naga_oil::compose::Composer,
    render_state: &render::state::BindGroupsCreated,
) -> Result<wgpu::RenderPipeline, naga_oil::compose::ComposerError> {
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/util.wgsl"),
        file_path: "shaders/util.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/vertex_fetch.wgsl"),
        file_path: "shaders/vertex_fetch.wgsl",
        ..Default::default()
    })?;

    let module = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("../shaders/object.wgsl"),
        file_path: "shaders/object.wgsl",
        ..Default::default()
    })?;
    let module = std::borrow::Cow::Owned(module);

    let shader = render_state
        .wgpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("object render pipeline"),
            source: wgpu::ShaderSource::Naga(module),
        });

    let layout = render_state
        .wgpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("object render pipeline layout"),
            bind_group_layouts: &[
                &render_state.bind_groups.object_data,
                &render_state.bind_groups.materials,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..64,
            }],
        });

    Ok(render_state
        .wgpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("object render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[render::MeshInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: render::TextureFormat::GBUFFER.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: render::TextureFormat::GBUFFER.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: render::TextureFormat::GBUFFER.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: render::TextureFormat::GBUFFER.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
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
        }))
}
