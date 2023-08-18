#![warn(rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::too_many_lines)]

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("wormhole")
        .build(&event_loop)
        .expect("failed to create window");

    let mut input = winit_input_helper::WinitInputHelper::new();

    // SAFETY:
    // This function is unsafe because the window must be valid as long as the surface is valid.
    // Because the surface is created after the window, the drop order ensures that the surface is dropped after the window.
    let mut render_state = unsafe { pollster::block_on(wormhole::render::State::new(&window)) };

    event_loop.run(move |event, _, control_flow| {
        // Process the event. Once the last event is processed, input.update will return true and we can execute our logic.
        if input.update(&event) {
            if let Some(size) = input.window_resized() {
                render_state.resize(size);
            }

            if input.close_requested() {
                control_flow.set_exit();
            }

            let output = match render_state.surface.get_current_texture() {
                Ok(texture) => texture,
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                    render_state.reconfigure_surface();

                    return;
                }
                Err(wgpu::SurfaceError::Timeout) => return,
                Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of gpu memory. exiting"),
            };

            let mut encoder =
                render_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("render pass encoder"),
                    });

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

            render_state.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });
}
