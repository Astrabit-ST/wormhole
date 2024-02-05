#![warn(rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::new_without_default, clippy::cast_possible_truncation)]

use bevy_ecs::{event::ManualEventReader, prelude::*, system::SystemState};
use winit::event::{DeviceEvent, Event, WindowEvent};

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
    let window = std::sync::Arc::new(window);

    #[cfg(feature = "capture_mouse")]
    if let Err(e) = window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
    {
        println!("error locking cursor {e}");
    }

    let render_state = pollster::block_on(wormhole::render::State::new(window.clone()));

    let mut scene = wormhole::scene::Scene::new(render_state);

    let mut event_writers_system_state: SystemState<wormhole::input::EventWriters<'_>> =
        SystemState::from_world(&mut scene.world);
    let mut exit_event_reader = ManualEventReader::<wormhole::input::Exit>::default();

    window.set_visible(true);
    #[cfg(feature = "capture_mouse")]
    window.set_cursor_visible(false);

    let result = event_loop.run(move |event, target| {
        //
        if let Some(exit_events) = scene.world.get_resource() {
            if exit_event_reader.read(exit_events).last().is_some() {
                target.exit();
            }
        }

        match event {
            Event::AboutToWait => {
                window.request_redraw();
                target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
            Event::WindowEvent { window_id, event } => {
                if window_id != window.id() {
                    return;
                }

                let mut event_writers: wormhole::input::EventWriters<'_> =
                    event_writers_system_state.get_mut(&mut scene.world);

                match event {
                    WindowEvent::Resized(size) => event_writers
                        .window_resized
                        .send(wormhole::input::WindowResized { size }),
                    WindowEvent::CloseRequested => event_writers
                        .close_requested
                        .send(wormhole::input::CloseRequested),
                    WindowEvent::KeyboardInput { event, .. } => {
                        event_writers.keyboard.send(wormhole::input::KeyboardEvent {
                            key_code: event.physical_key,
                            state: event.state,
                        });
                    }
                    WindowEvent::MouseInput { state, button, .. } => event_writers
                        .mouse_button
                        .send(wormhole::input::MouseButtonEvent { button, state }),
                    WindowEvent::MouseWheel { delta, .. } => event_writers
                        .mouse_wheel
                        .send(wormhole::input::MouseWheel { delta }),
                    WindowEvent::RedrawRequested => {
                        scene.update();
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                let mut event_writers: wormhole::input::EventWriters<'_> =
                    event_writers_system_state.get_mut(&mut scene.world);
                event_writers
                    .mouse_motion
                    .send(wormhole::input::MouseMotion {
                        delta: glam::vec2(x as f32, y as f32),
                    });
            }
            _ => {}
        }
    });
    if let Err(e) = result {
        eprintln!("event loop error {e}");
    }
}
