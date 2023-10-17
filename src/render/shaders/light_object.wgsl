#import wormhole::vertex_fetch as Fetch

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
}
var<push_constant> camera: Camera;

struct Transform {
    obj_proj: mat4x4<f32>,
    normal_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<storage> transforms: array<Transform>;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: Fetch::InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let transform = transforms[instance.transform_index];

    let model_position = Fetch::read_vertex_position(vertex_index, instance.position_offset);
    let world_position = transform.obj_proj * vec4<f32>(model_position, 1.0);

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

    out.color = vec4<f32>(1.0);

    return out;
}
