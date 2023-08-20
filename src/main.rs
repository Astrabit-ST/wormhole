#![warn(rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::new_without_default)]

fn main() {
    color_backtrace::install();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("wormhole")
        .with_visible(false)
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("failed to create window");

    if let Err(e) = window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
    {
        println!("error locking cursor {e}");
    }

    // SAFETY:
    // This function is unsafe because the window must be valid as long as the surface is valid.
    // Because the surface is created after the window, the drop order ensures that the surface is dropped after the window.
    let mut render_state = unsafe { pollster::block_on(wormhole::render::State::new(&window)) };
    render_state.initialize_bind_group_layouts();
    render_state.initialize_shaders();

    let mut input_state = wormhole::input::State::new();

    window.set_visible(true);
    window.set_cursor_visible(false);

    let mut camera = wormhole::render::Camera::new(&render_state);
    let object = wormhole::object::Object::new(&render_state);

    event_loop.run(move |event, _, control_flow| {
        // Process the event. Once the last event is processed, input.process will return true and we can execute our logic.
        if input_state.process(&event) {
            if let Some(size) = input_state.new_window_size() {
                render_state.resize(size);
            }

            if input_state.close_requested() {
                control_flow.set_exit();
            }

            if input_state
                .keyboard
                .pressed(winit::event::VirtualKeyCode::Escape)
            {
                control_flow.set_exit();
            }

            camera.update(&render_state, &input_state);

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

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            object.draw(&camera, &mut render_pass);

            drop(render_pass);

            render_state.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    });
}