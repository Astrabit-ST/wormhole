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

fn extract_flag(data: u32, flag: u32) -> bool {
    return bool(data & flag);
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct Camera {
    viewport_size: vec2<f32>,
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct Transform {
    obj_proj: mat4x4<f32>,
    normal_proj: mat4x4<f32>,
}

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

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<storage> transforms: array<Transform>;

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

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let transform = transforms[instance.transform_index];

    let model_position = read_vertex_position(vertex_index, instance.position_offset);
    let world_position = transform.obj_proj * vec4<f32>(model_position, 1.0);

    out.clip_position = camera.view_proj * world_position;

    return out;
}

struct Constants {
    color: vec4<f32>,
}
var<push_constant> constants: Constants;

// Fragment shader
struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = constants.color;

    return out;
}
