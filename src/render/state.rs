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
use crate::light;
use crate::object;
use crate::render;

pub struct GpuCreated {
    pub wgpu: GpuState,
}

pub struct BindGroupsCreated {
    pub wgpu: GpuState,
    pub bind_groups: BindGroups,
}

pub struct State {
    pub wgpu: GpuState,
    pub bind_groups: BindGroups,
    pub pipelines: RenderPipelines,
}

pub struct GpuState {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct BindGroups {
    pub camera: wgpu::BindGroupLayout,
    pub transform: wgpu::BindGroupLayout,
    pub light: wgpu::BindGroupLayout,
    pub object_textures: wgpu::BindGroupLayout,
    pub gbuffer: wgpu::BindGroupLayout,
}

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
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("wgpu device"),
                    limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        max_buffer_size: adapter.limits().max_buffer_size,
                        ..Default::default()
                    },
                    features: wgpu::Features::PUSH_CONSTANTS,
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
        use render::traits::Bindable;

        let camera = render::Camera::create_bind_group_layout(&self);
        let transform = render::Transform::create_bind_group_layout(&self);
        let light = light::PreparedLight::create_bind_group_layout(&self);
        let object_textures = object::Textures::create_bind_group_layout(&self);
        let gbuffer = render::buffer::geometry::Buffer::create_bind_group_layout(&self);

        BindGroupsCreated {
            wgpu: self.wgpu,
            bind_groups: BindGroups {
                camera,
                transform,
                light,
                object_textures,
                gbuffer,
            },
        }
    }
}

impl BindGroupsCreated {
    /// Initializes the bind group layouts of all uniforms passed to shaders.
    /// Call this before initializing shaders, as they are dependent on these layouts.
    pub fn initialize_render_pipelines(self) -> State {
        let object = object::Object::create_render_pipeline(&self);
        let light = light::Light::create_light_render_pipeline(&self);
        let light_object = light::Light::create_light_object_render_pipeline(&self);

        State {
            wgpu: self.wgpu,
            bind_groups: self.bind_groups,
            pipelines: RenderPipelines {
                object,
                light,
                light_object,
            },
        }
    }
}

impl State {
    pub fn reconfigure_surface(&self) {
        self.wgpu
            .surface
            .configure(&self.wgpu.device, &self.wgpu.surface_config)
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.wgpu.surface_config.width = size.width;
            self.wgpu.surface_config.height = size.height;
            self.reconfigure_surface();
        }
    }
}
