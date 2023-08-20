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

    let mut scene = wormhole::scene::Scene::new(&render_state);

    window.set_visible(true);
    window.set_cursor_visible(false);

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

            scene.update(&render_state, &input_state);

            scene.render(&render_state);
        }
    });
}
