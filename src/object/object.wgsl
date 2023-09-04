// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) position: vec3<f32>
};

struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Transform {
    obj_proj: mat4x4<f32>
}
@group(1) @binding(0)
var<storage> transforms: array<Transform>;

struct Constants {
    transform_index: u32
}
var<push_constant> constants: Constants;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let transform = transforms[constants.transform_index];

    out.tex_coords = model.tex_coords;
    out.normal = (transform.obj_proj * vec4<f32>(model.normal, 0.0)).xyz;
    out.position = (transform.obj_proj * vec4<f32>(model.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * transform.obj_proj * vec4<f32>(model.position, 1.0);

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

    let normal = vec4<f32>(in.normal, 0.0) * textureSample(t_normal, s_normal, in.tex_coords);

    out.albedo = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    out.normal = normal;
    out.position = vec4<f32>(in.position, 0.0);

    return out;
}
