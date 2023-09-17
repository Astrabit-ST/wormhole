struct Constants {
    transform_index: i32,

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,

    position: vec3<f32>,
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

@group(2) @binding(0)
var g_albedo: texture_2d<f32>;
@group(2) @binding(1)
var s_albedo: sampler;

@group(2) @binding(2)
var g_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;

@group(2) @binding(4)
var g_position: texture_2d<f32>;
@group(2) @binding(5)
var s_position: sampler;

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

    let screen_coord = in.clip_position.xy / camera.viewport_size;

    let frag_pos = textureSample(g_position, s_position, screen_coord).rgb;
    let normal = textureSample(g_normal, s_normal, screen_coord).rgb;
    let albedo = textureSample(g_albedo, s_albedo, screen_coord).rgb;

    let ambient = constants.ambient.rgb * albedo;

    let light_dir = normalize(constants.position - frag_pos);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse = constants.diffuse.rgb * diffuse_strength * albedo;

    let view_dir = normalize(camera.view_pos.xyz - frag_pos);
    let half_dir = normalize(view_dir + light_dir);
    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular = constants.specular.rgb * specular_strength;

    let distance = length(constants.position - frag_pos);
    let attenuation = 1.0 / (constants.constant + constants.linear * distance + constants.quadratic * (distance * distance));

    let ambient_color = ambient * attenuation;
    let diffuse_color = diffuse * attenuation;
    let specular_color = specular * attenuation;

    let result = ambient_color + diffuse_color + specular_color;
    out.color = vec4<f32>(result, 1.0);

    return out;
}
