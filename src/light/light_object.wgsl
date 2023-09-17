struct Constants {
    transform_index: i32,

    color: vec4<f32>,
}
var<push_constant> constants: Constants;

struct Camera {
    viewport_size: vec2<f32>,
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
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let transform = transforms[constants.transform_index];

    let world_position = transform.obj_proj * vec4<f32>(model.position, 1.0);

    out.clip_position = camera.view_proj * world_position;

    return out;
}

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
