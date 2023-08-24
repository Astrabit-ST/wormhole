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
use crate::input;
use winit::event::{Event, WindowEvent};

pub struct State {
    pub keyboard: input::Keyboard,
    pub mouse: input::Mouse,
    pub controller: input::Controller,

    close_requested: bool,
    new_size: Option<winit::dpi::PhysicalSize<u32>>,
    window_id: winit::window::WindowId,
    monitor_resolution: winit::dpi::PhysicalSize<u32>,
}

impl State {
    pub fn new(window: &winit::window::Window) -> Self {
        let keyboard = input::Keyboard::new();
        let mouse = input::Mouse::new();
        let controller = input::Controller::new();

        let monitor_resolution = window
            .current_monitor()
            .as_ref()
            .map(winit::monitor::MonitorHandle::size)
            .unwrap_or_default();

        Self {
            keyboard,
            mouse,
            controller,

            close_requested: false,
            new_size: None,
            window_id: window.id(),
            monitor_resolution,
        }
    }

    // notify everything that we're starting a new frame and set frame specific variables.
    pub fn start_frame(&mut self) {
        self.keyboard.start_frame();
        self.mouse.start_frame();

        self.new_size = None;
        self.close_requested = false;
    }

    pub fn process<T>(
        &mut self,
        event: &winit::event::Event<'_, T>,
        window: &winit::window::Window,
    ) -> bool {
        match event {
            Event::NewEvents(_) => self.start_frame(),
            Event::WindowEvent { event, window_id } if *window_id == self.window_id => {
                match event {
                    WindowEvent::Resized(size) => self.new_size = Some(*size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.new_size = Some(**new_inner_size)
                    }
                    // if the window has moved, it might have moved to another monitor.
                    WindowEvent::Moved(_) => {
                        if let Some(monitor) = window.current_monitor() {
                            self.monitor_resolution = monitor.size();
                        }
                    }
                    WindowEvent::CloseRequested => self.close_requested = true,
                    event => {
                        self.keyboard.process(event);
                        self.mouse.process_window(event);
                    }
                }
            }
            Event::DeviceEvent { event, .. } => {
                self.mouse.process_device(event);
            }
            _ => {}
        }
        matches!(event, winit::event::Event::MainEventsCleared)
    }

    pub fn move_direction(&self) -> glam::Vec2 {
        let mut vector = glam::Vec2::ZERO;

        if self.keyboard.held(winit::event::VirtualKeyCode::W) {
            vector.y += 1.0;
        }

        if self.keyboard.held(winit::event::VirtualKeyCode::A) {
            vector.x -= 1.0;
        }

        if self.keyboard.held(winit::event::VirtualKeyCode::S) {
            vector.y -= 1.0;
        }

        if self.keyboard.held(winit::event::VirtualKeyCode::D) {
            vector.x -= 1.0;
        }

        vector
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub fn new_window_size(&self) -> Option<winit::dpi::PhysicalSize<u32>> {
        self.new_size
    }

    pub fn monitor_resolution(&self) -> winit::dpi::PhysicalSize<u32> {
        self.monitor_resolution
    }
}
