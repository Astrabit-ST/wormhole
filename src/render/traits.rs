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

pub trait Bindable {
    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static>;

    fn create_bind_group_layout(render_state: &render::state::GpuCreated) -> wgpu::BindGroupLayout {
        render_state
            .wgpu
            .device
            .create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR)
    }
}

pub trait DynamicBufferWriteable:
    encase::ShaderSize + encase::ShaderType + encase::internal::WriteInto
{
    const ALIGN: u64 = 32;

    fn get_layout(render_state: &render::State) -> &wgpu::BindGroupLayout;
}

pub trait Shadeable {
    fn create_render_pipeline(
        render_state: &render::state::BindGroupsCreated,
    ) -> wgpu::RenderPipeline;
}