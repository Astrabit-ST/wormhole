// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,

    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,

    @location(0) tex_coords: vec2<f32>,
    @location(1) position: vec3<f32>,

    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
};

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Transform {
    obj_proj: mat4x4<f32>,
    normal_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<storage> transforms: array<Transform>;

struct Constants {
    transform_index: i32
}
var<push_constant> constants: Constants;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let transform = transforms[constants.transform_index];

    let world_position = transform.obj_proj * vec4<f32>(model.position, 1.0);

    out.tex_coords = model.tex_coords;

    out.position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;

    let normal_matrix = mat3x3<f32>(transform.normal_proj[0].xyz, transform.normal_proj[1].xyz, transform.normal_proj[2].xyz);

    out.world_normal = normalize(normal_matrix * model.normal);
    out.world_tangent = normalize(normal_matrix * model.tangent);
    out.world_bitangent = normalize(normal_matrix * model.bitangent);

    return out;
}

// Fragment shader

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) position: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let tangent_matrix = mat3x3<f32>(
        in.world_tangent,
        in.world_bitangent,
        in.world_normal,
    );

    var normal_rgb = textureSample(t_normal, s_normal, in.tex_coords).xyz;
    let normal_map = normal_rgb * 2.0 - 1.0;
    let normal = normalize(tangent_matrix * normal_rgb);

    out.albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    out.normal = vec4<f32>(in.world_normal, 0.0);
    out.position = vec4<f32>(in.position, 0.0);

    return out;
}
