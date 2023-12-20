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

const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
const SCREEN_VERTEX_DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: 20 as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: ATTRS,
};

pub fn create_light_object_render_pipeline(
    composer: &mut naga_oil::compose::Composer,
    render_state: &render::state::BindGroupsCreated,
) -> Result<wgpu::RenderPipeline, naga_oil::compose::ComposerError> {
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("util.wgsl"),
        file_path: "util.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("vertex_fetch.wgsl"),
        file_path: "vertex_fetch.wgsl",
        ..Default::default()
    })?;

    let module = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("light_object.wgsl"),
        file_path: "light_object.wgsl",
        ..Default::default()
    })?;
    let module = std::borrow::Cow::Owned(module);

    let shader = render_state
        .wgpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("light object render pipeline"),
            source: wgpu::ShaderSource::Naga(module),
        });

    let layout = render_state
        .wgpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("light object render pipeline layout"),
            bind_group_layouts: &[&render_state.bind_groups.object_data],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..64,
            }],
        });

    Ok(render_state
        .wgpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("light object render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[render::MeshInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.wgpu.surface_config.lock().format,
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

pub fn create_light_render_pipeline(
    composer: &mut naga_oil::compose::Composer,
    render_state: &render::state::BindGroupsCreated,
) -> Result<wgpu::RenderPipeline, naga_oil::compose::ComposerError> {
    let module = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("light.wgsl"),
        file_path: "light.wgsl",
        ..Default::default()
    })?;
    let module = std::borrow::Cow::Owned(module);

    let shader = render_state
        .wgpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("light render pipeline"),
            source: wgpu::ShaderSource::Naga(module),
        });

    let layout = render_state
        .wgpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lighting render pipeline layout"),
            bind_group_layouts: &[
                &render_state.bind_groups.light_data,
                &render_state.bind_groups.gbuffer,
            ],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..16,
            }],
        });

    Ok(render_state
        .wgpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("light render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SCREEN_VERTEX_DESC], // FIXME
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.wgpu.surface_config.lock().format,
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
        }))
}
