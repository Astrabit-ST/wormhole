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

use crate::{render, shaders};

use bevy_ecs::prelude::*;

use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Resource)]
#[derive(Debug, Clone)]
pub struct State {
    pub wgpu: Arc<GpuState>,
    pub bind_groups: Arc<BindGroups>,
    pub pipelines: Arc<RenderPipelines>,
}

pub struct GpuCreated {
    pub wgpu: GpuState,
}

pub struct BindGroupsCreated {
    pub wgpu: GpuState,
    pub bind_groups: BindGroups,
}

#[derive(Debug)]
pub struct GpuState {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    // augh mutation behind an Arc SUCKS
    // todo maybe convert the sizes into atomics instead? we only update this when resizing the window
    pub surface_config: Mutex<wgpu::SurfaceConfiguration>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Debug)]
pub struct BindGroups {
    pub object_data: wgpu::BindGroupLayout,
    pub materials: wgpu::BindGroupLayout,
    pub gbuffer: wgpu::BindGroupLayout,
    pub light_data: wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub struct RenderPipelines {
    pub object: wgpu::RenderPipeline,
    pub light: wgpu::RenderPipeline,
    pub light_object: wgpu::RenderPipeline,
}

impl GpuCreated {
    /// # Safety
    ///
    /// See [`wgpu::Instance::create_surface`] for how to use this function safely.
    pub async unsafe fn new(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(), // FIXME: support up-to-date DX12 compiler
            flags: wgpu::InstanceFlags::from_build_config(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("failed to create window")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("failed to create adapter");

        let info = adapter.get_info();
        log::info!("Backend : {:?}", info.backend);
        log::info!("Device  : {}", info.name);
        log::info!("Driver  : {} {}", info.driver, info.driver_info);

        let adapter_limits = adapter.limits();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("wgpu device"),
                    limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        max_sampled_textures_per_shader_stage: adapter_limits
                            .max_sampled_textures_per_shader_stage,
                        ..Default::default()
                    },
                    features: wgpu::Features::PUSH_CONSTANTS
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::INDIRECT_FIRST_INSTANCE
                        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING // TODO: do we need this?
                        | wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY
                        | wgpu::Features::MULTI_DRAW_INDIRECT,
                },
                None,
            )
            .await
            .expect("failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let surface_config = Mutex::new(surface_config);

        let wgpu = GpuState {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
        };
        Self { wgpu }
    }

    /// Initializes the bind group layouts of all uniforms passed to shaders.
    /// Call this before initializing shaders, as they are dependent on these layouts.
    pub fn initialize_bind_group_layouts(self) -> BindGroupsCreated {
        const GENERIC_STORAGE: wgpu::BindingType = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        const GENERIC_TEXTURE: wgpu::BindingType = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };
        const GENERIC_SAMPLER: wgpu::BindingType =
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering);

        let object_data = render::BindGroupLayoutBuilder::new()
            // transforms
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            // vertex positions
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            // vertex normals
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            // vertex tex_coords
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            // vertex colors
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            // vertex tangents
            .append(wgpu::ShaderStages::VERTEX, GENERIC_STORAGE, None)
            .build(
                &self.wgpu.device,
                Some("wormhole object data bind group layout"),
            );

        let materials = render::BindGroupLayoutBuilder::new()
            // Sampler
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_SAMPLER, None)
            // Textures
            .append(
                wgpu::ShaderStages::FRAGMENT,
                GENERIC_TEXTURE,
                // Limit size to the max sampled textures per shader stage
                std::num::NonZeroU32::new(
                    self.wgpu
                        .device
                        .limits()
                        .max_sampled_textures_per_shader_stage,
                ),
            )
            // Material data
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_STORAGE, None)
            .build(
                &self.wgpu.device,
                Some("wormhole material data bind group layout"),
            );

        let gbuffer = render::BindGroupLayoutBuilder::new()
            // Sampler
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_SAMPLER, None)
            // Color + roughness
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_TEXTURE, None)
            // Normal + Metallicity
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_TEXTURE, None)
            // Position + Occlusion
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_TEXTURE, None)
            // Emissive
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_TEXTURE, None)
            .build(
                &self.wgpu.device,
                Some("wormhole gbuffer bind group layout"),
            );

        let light_data = render::BindGroupLayoutBuilder::new()
            .append(wgpu::ShaderStages::FRAGMENT, GENERIC_STORAGE, None)
            .build(
                &self.wgpu.device,
                Some("wormhole light data bind group layout"),
            );

        BindGroupsCreated {
            wgpu: self.wgpu,
            bind_groups: BindGroups {
                object_data,
                materials,
                gbuffer,
                light_data,
            },
        }
    }
}

impl BindGroupsCreated {
    /// Initializes the bind group layouts of all uniforms passed to shaders.
    /// Call this before initializing shaders, as they are dependent on these layouts.
    pub fn initialize_render_pipelines(self) -> State {
        let mut composer = naga_oil::compose::Composer::default()
            .with_capabilities(wgpu::naga::valid::Capabilities::PUSH_CONSTANT | wgpu::naga::valid::Capabilities::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING);
        let object = match shaders::object::create_render_pipeline(&mut composer, &self) {
            Ok(p) => p,
            Err(err) => {
                let err = err.emit_to_string(&composer);
                panic!("Error creating object render pipeline:\n{err}")
            }
        };
        let light = match shaders::light::create_light_render_pipeline(&mut composer, &self) {
            Ok(p) => p,
            Err(err) => {
                let err = err.emit_to_string(&composer);
                panic!("Error creating light render pipeline:\n{err}")
            }
        };
        let light_object =
            match shaders::light::create_light_object_render_pipeline(&mut composer, &self) {
                Ok(p) => p,
                Err(err) => {
                    let err = err.emit_to_string(&composer);
                    panic!("Error creating light object render pipeline:\n{err}")
                }
            };

        State {
            wgpu: Arc::new(self.wgpu),
            bind_groups: Arc::new(self.bind_groups),
            pipelines: Arc::new(RenderPipelines {
                object,
                light,
                light_object,
            }),
        }
    }
}

impl State {
    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            let mut config = self.wgpu.surface_config.lock();
            config.width = size.width;
            config.height = size.height;

            self.wgpu.surface.configure(&self.wgpu.device, &config)
        }
    }
}
