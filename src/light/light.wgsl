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
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;

    return out;
}

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
const light: Light = Light(vec3<f32>(0.0, 2.0, 0.0), vec3<f32>(0.6, 0.65, 1.0));

// Fragment shader

@group(1) @binding(0)
var g_albedo: texture_2d<f32>;
@group(1) @binding(1)
var s_albedo: sampler;

@group(1) @binding(2)
var g_normal: texture_2d<f32>;
@group(1) @binding(3)
var s_normal: sampler;

@group(1) @binding(4)
var g_position: texture_2d<f32>;
@group(1) @binding(5)
var s_position: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_pos = textureSample(g_position, s_position, in.tex_coords).rgb;
    let normal = textureSample(g_normal, s_normal, in.tex_coords).rgb;
    let albedo = textureSample(g_albedo, s_albedo, in.tex_coords).rgb;

    let ambient_strength = 0.01;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - frag_pos);

    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(camera.view_pos.xyz - frag_pos);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * albedo.xyz;

    return vec4<f32>(result, 1.0);
}
