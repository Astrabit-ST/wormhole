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

#[allow(dead_code)]
pub struct Textures {
    bind_group: wgpu::BindGroup,
}

static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

impl Textures {
    pub fn new(render_state: &render::State, albedo: &render::Texture) -> Self {
        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("object textures bind group"),
                layout: Textures::bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&albedo.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&albedo.sampler),
                    },
                ],
            });

        Self { bind_group }
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>, index: u32) {
        render_pass.set_bind_group(index, &self.bind_group, &[]);
    }
}

impl Textures {
    pub fn create_bind_group_layout(render_state: &render::State) {
        let layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("object texture bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        LAYOUT
            .set(layout)
            .expect("object texture bind group layout");
    }

    pub fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        LAYOUT.get().expect("layout uninitialized")
    }
}
