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

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

#[derive(encase::ShaderType, Debug)]
pub struct Data {
    obj_proj: glam::Mat4,
    normal_proj: glam::Mat4,
}

impl Transform {
    fn to_obj_proj(self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    fn to_normal_proj(self) -> glam::Mat4 {
        glam::Mat4::from_quat(self.rotation)
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_position(glam::vec3(x, y, z))
    }

    pub fn from_position(position: glam::Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn from_position_scale(position: glam::Vec3, scale: glam::Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::IDENTITY,
            scale,
        }
    }

    pub fn from_position_rotation(position: glam::Vec3, rotation: glam::Quat) -> Self {
        Self {
            position,
            rotation,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn from_gltf(transform: gltf::scene::Transform) -> Self {
        let (position, rotation, scale) = transform.decomposed();
        Self {
            position: glam::Vec3::from_array(position),
            rotation: glam::Quat::from_array(rotation),
            scale: glam::Vec3::from_array(scale),
        }
    }
}

impl From<gltf::scene::Transform> for Transform {
    fn from(value: gltf::scene::Transform) -> Self {
        Self::from_gltf(value)
    }
}

impl encase::ShaderSize for Transform {}

impl encase::ShaderType for Transform {
    type ExtraMetadata = <Data as encase::ShaderType>::ExtraMetadata;
    const METADATA: encase::private::Metadata<Self::ExtraMetadata> =
        <Data as encase::ShaderType>::METADATA;
}

impl encase::internal::WriteInto for Transform {
    fn write_into<B>(&self, writer: &mut encase::internal::Writer<B>)
    where
        B: encase::internal::BufferMut,
    {
        let obj_proj = self.to_obj_proj();
        let normal_proj = self.to_normal_proj();
        let data = Data {
            obj_proj,
            normal_proj,
        };
        data.write_into(writer)
    }
}
