// Vertex shader

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

    @location(0) tex_coords: vec2<f32>,
    @location(1) position: vec3<f32>,

    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
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

    let tex_coords = read_vertex_tex_coords(vertex_index, instance.tex_coord_offset);
    out.tex_coords = tex_coords;

    out.position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;

    let normal_matrix = mat3x3<f32>(transform.normal_proj[0].xyz, transform.normal_proj[1].xyz, transform.normal_proj[2].xyz);

    let model_normal = read_vertex_normal(vertex_index, instance.normal_offset);
    let model_tangent = read_vertex_tangent(vertex_index, instance.tangent_offset);
    let model_bitangent = cross(model_normal, model_tangent.xyz) * model_tangent.w;

    out.world_normal = normalize(normal_matrix * model_normal);
    out.world_tangent = normalize(normal_matrix * model_tangent.xyz);
    out.world_bitangent = normalize(normal_matrix * model_bitangent);

    return out;
}

// Fragment shader

struct Material {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    metallic: f32,
    roughness: f32,
    flags: u32,
}

const HAS_BASE_COLOR_TEXTURE         = 0x0001u;
const HAS_METALLIC_ROUGHNESS_TEXTURE = 0x0002u;
const HAS_EMISSIVE_TEXTURE           = 0x0004u;
const HAS_OCCLUSION_TEXTURE          = 0x0008u;
const HAS_NORMAL_MAP                 = 0x0010u;

@group(3) @binding(0)
var<uniform> material: Material;

@group(3) @binding(1)
var t_color: texture_2d<f32>;
@group(3) @binding(2)
var s_color: sampler;

@group(3) @binding(3)
var t_normal: texture_2d<f32>;
@group(3) @binding(4)
var s_normal: sampler;

@group(3) @binding(5)
var t_metallic_roughness: texture_2d<f32>;
@group(3) @binding(6)
var s_metallic_roughness: sampler;

@group(3) @binding(7)
var t_emissive: texture_2d<f32>;
@group(3) @binding(8)
var s_emissive: sampler;

@group(3) @binding(9)
var t_occlusion: texture_2d<f32>;
@group(3) @binding(10)
var s_occlusion: sampler;

struct FragmentOutput {
    @location(0) color_roughness: vec4<f32>,
    @location(1) normal_metallicity: vec4<f32>,
    @location(2) position_occlusion: vec4<f32>,
    @location(3) emissive: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let base_color_texture = textureSample(t_color, s_color, in.tex_coords).rgb;
    let normal_map_texture = textureSample(t_normal, s_normal, in.tex_coords).rgb;
    let metallic_roughness_texture = textureSample(t_metallic_roughness, s_metallic_roughness, in.tex_coords);
    let emissive_texture = textureSample(t_emissive, s_emissive, in.tex_coords);
    let occlusion_texture = textureSample(t_occlusion, s_occlusion, in.tex_coords);

    var base_color = material.base_color.rgb;
    if extract_flag(material.flags, HAS_BASE_COLOR_TEXTURE) {
        base_color = base_color_texture;
    }

    var normal = in.world_normal;
    if extract_flag(material.flags, HAS_NORMAL_MAP) {
        let tangent_matrix = mat3x3<f32>(
            in.world_tangent,
            in.world_bitangent,
            in.world_normal,
        );
        let normal_map = normal_map_texture * 2.0 - 1.0;
        normal = normalize(tangent_matrix * normal_map);
    }

    var metallicity = material.metallic;
    var roughness = material.roughness;
    if extract_flag(material.flags, HAS_METALLIC_ROUGHNESS_TEXTURE) {
        metallicity = metallic_roughness_texture.b;
        roughness = metallic_roughness_texture.g;
    }

    var emissive = material.emissive.xyz;
    if extract_flag(material.flags, HAS_EMISSIVE_TEXTURE) {
        emissive = emissive_texture.xyz;
    }

    var occlusion = 0.0;
    if extract_flag(material.flags, HAS_OCCLUSION_TEXTURE) {
        occlusion = occlusion_texture.r;
    }

    out.color_roughness = vec4<f32>(base_color, roughness);
    out.normal_metallicity = vec4<f32>(in.world_normal, metallicity);
    out.position_occlusion = vec4<f32>(in.position, occlusion);
    out.emissive = vec4<f32>(emissive, 1.0);

    return out;
}
