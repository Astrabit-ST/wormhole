#![warn(rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::new_without_default)]

fn main() {
    color_backtrace::install();
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new().expect("failed to create event loop");
    let window = winit::window::WindowBuilder::new()
        .with_title("wormhole")
        .with_visible(false)
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .expect("failed to create window");

    #[cfg(feature = "capture_mouse")]
    if let Err(e) = window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
    {
        println!("error locking cursor {e}");
    }

    // SAFETY:
    // This function is unsafe because the window must be valid as long as the surface is valid.
    // Because the surface is created after the window, the drop order ensures that the surface is dropped after the window.
    let render_state =
        unsafe { pollster::block_on(wormhole::render::state::GpuCreated::new(&window)) }
            .initialize_bind_group_layouts()
            .initialize_render_pipelines();

    let mut scene = wormhole::scene::Scene::new(render_state, &window);

    window.set_visible(true);
    #[cfg(feature = "capture_mouse")]
    window.set_cursor_visible(false);

    let result = event_loop.run(move |event, target| {
        target.set_control_flow(winit::event_loop::ControlFlow::Poll);
        // Process the event.
        scene.process_event(&event, target, &window);
    });
    if let Err(e) = result {
        eprintln!("event loop error {e}");
    }
}
