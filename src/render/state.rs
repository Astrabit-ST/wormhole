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

pub struct State {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl State {
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
                    limits: wgpu::Limits::default(),
                    features: wgpu::Features::default(),
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

        Self {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
        }
    }

    /// Call this after initializing bind group layouts.
    /// Failure to do so will result in a panic.
    // FIXME: enforce this behaviour with generics?
    pub fn initialize_shaders(&self) {
        crate::object::Shader::create(self);
    }

    /// Initializes the bind group layouts of all uniforms passed to shaders.
    /// Call this before initializing shaders, as they are dependent on these layouts.
    pub fn initialize_bind_group_layouts(&self) {
        super::Camera::create_bind_group_layout(self);
        super::Transform::create_bind_group_layout(self);
        crate::object::Textures::create_bind_group_layout(self);
    }

    pub fn reconfigure_surface(&self) {
        self.surface.configure(&self.device, &self.surface_config)
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.reconfigure_surface();
    }
}
