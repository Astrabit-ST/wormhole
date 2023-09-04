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
use crate::render;

pub struct GBuffer {
    pub albedo: render::Texture,
    pub normal: render::Texture,
    pub position: render::Texture,

    pub depth: render::Texture,
}

impl GBuffer {
    pub fn new(render_state: &render::State) -> Self {
        let albedo = render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let normal = render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);
        let position =
            render::Texture::new_screen_size(render_state, render::TextureFormat::GBUFFER);

        let depth = render::Texture::new_screen_size(render_state, render::TextureFormat::DEPTH);

        Self {
            albedo,
            normal,
            position,

            depth,
        }
    }

    pub fn resize_to_screen(&mut self, render_state: &render::State) {
        self.albedo.resize_to_screen(render_state);
        self.normal.resize_to_screen(render_state);
        self.position.resize_to_screen(render_state);

        self.depth.resize_to_screen(render_state);
    }

    pub fn as_color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 3] {
        [
            Some(wgpu::RenderPassColorAttachment {
                view: &self.albedo.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.position.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }),
        ]
    }

    pub fn depth_stencil_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> {
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        })
    }
}
