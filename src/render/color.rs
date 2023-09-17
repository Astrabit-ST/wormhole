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

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub color: glam::Vec4,
}

impl Color {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_rgba(color: glam::Vec4) -> Self {
        Self::from_rgba_normalized(color / 255.)
    }

    pub fn from_rgba_normalized(color: glam::Vec4) -> Self {
        Self { color }
    }

    pub fn from_rgb(color: glam::Vec3) -> Self {
        Self::from_rgba(color.extend(1.0))
    }

    pub fn from_rgb_normalized(color: glam::Vec3) -> Self {
        Self::from_rgba_normalized(color.extend(1.0))
    }
}

impl From<glam::Vec4> for Color {
    fn from(value: glam::Vec4) -> Self {
        Self::from_rgba(value)
    }
}

impl From<Color> for glam::Vec4 {
    fn from(value: Color) -> Self {
        value.color
    }
}

impl From<glam::Vec3> for Color {
    fn from(value: glam::Vec3) -> Self {
        Self::from_rgb(value)
    }
}

impl From<Color> for glam::Vec3 {
    fn from(value: Color) -> Self {
        value.color.truncate()
    }
}
