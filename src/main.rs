#![warn(rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::too_many_lines)]

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("wormhole")
        .build(&event_loop)
        .expect("failed to create window");

    let mut input = winit_input_helper::WinitInputHelper::new();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
        dx12_shader_compiler: wgpu::Dx12Compiler::default(), // FIXME: support up-to-date DX12 compiler
    });
    // SAFETY:
    // This function is unsafe because the window must be valid as long as the surface is valid.
    // Because the surface is created after the window, the drop order ensures that the surface is dropped after the window.
    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("failed to create window")
    };
    let (adapter, device, queue) = pollster::block_on(async {
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

        (adapter, device, queue)
    });

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .unwrap_or(surface_caps.formats[0]);
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    event_loop.run(move |event, _, control_flow| {
        // Process the event. Once the last event is processed, input.update will return true and we can execute our logic.
        if input.update(&event) {
            if let Some(size) = input.window_resized() {
                surface_config.width = size.width;
                surface_config.height = size.height;

                surface.configure(&device, &surface_config);
            }

            if input.close_requested() {
                control_flow.set_exit();
            }

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render pass encoder"),
            });

            let output = match surface.get_current_texture() {
                Ok(texture) => texture,
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                    surface.configure(&device, &surface_config);

                    return;
                }
                Err(wgpu::SurfaceError::Timeout) => return,
                Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of gpu memory. exiting"),
            };
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("wormhole main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 0.,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            drop(render_pass);

            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });
}
