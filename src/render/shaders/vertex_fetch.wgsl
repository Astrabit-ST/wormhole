#define_import_path wormhole::vertex_fetch

#import wormhole::util

struct InstanceInput {
    @location(0) position_offset: u32,
    @location(1) normal_offset: u32,
    @location(2) tex_coord_offset: u32,
    @location(3) color_offset: u32,
    @location(4) tangent_offset: u32,

    @location(5) format_flags: u32,

    @location(6) transform_index: u32,
}

const HAS_VTX_NORMALS   = 0x0001u;
const HAS_TEX_COORDS    = 0x0002u;
const HAS_VTX_COLOR     = 0x0004u;
const HAS_VTX_TANGENT   = 0x0008u;

@group(0) @binding(0)
var<storage> position_data: array<f32>;
@group(0) @binding(1)
var<storage> normal_data: array<f32>;
@group(0) @binding(2)
var<storage> tex_coord_data: array<f32>;
@group(0) @binding(3)
var<storage> color_data: array<f32>;
@group(0) @binding(4)
var<storage> tangent_data: array<f32>;

fn read_vertex_position(vertex_index: u32, byte_offset: u32) -> vec3<f32> {
    let first_element_offset = byte_offset / 4u + vertex_index * 3u;
    return vec3<f32>(
        position_data[ first_element_offset],
        position_data[ first_element_offset + 1u],
        position_data[ first_element_offset + 2u],
    );
}

fn read_vertex_tex_coords(vertex_index: u32, byte_offset: u32) -> vec2<f32> {
    let first_element_offset = byte_offset / 4u + vertex_index * 2u;
    return vec2<f32>(
        tex_coord_data[ first_element_offset],
        tex_coord_data[ first_element_offset + 1u]
    );
}

fn read_vertex_normal(vertex_index: u32, byte_offset: u32) -> vec3<f32> {
    let first_element_offset = byte_offset / 4u + vertex_index * 3u;
    return vec3<f32>(
        normal_data[ first_element_offset],
        normal_data[ first_element_offset + 1u],
        normal_data[ first_element_offset + 2u],
    );
}

fn read_vertex_tangent(vertex_index: u32, byte_offset: u32) -> vec4<f32> {
    let first_element_offset = byte_offset / 4u + vertex_index * 4u;
    return vec4<f32>(
        tangent_data[first_element_offset],
        tangent_data[first_element_offset + 1u],
        tangent_data[first_element_offset + 2u],
        tangent_data[first_element_offset + 3u],
    );
}

fn read_vertex_color(vertex_index: u32, byte_offset: u32) -> vec4<f32> {
    let first_element_offset = byte_offset / 4u + vertex_index * 4u;
    return vec4<f32>(
        color_data[first_element_offset],
        color_data[first_element_offset + 1u],
        color_data[first_element_offset + 2u],
        color_data[first_element_offset + 3u],
    );
}