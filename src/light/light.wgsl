// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
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

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
const light: Light = Light(vec3<f32>(0.0, 2.0, 0.0), vec3<f32>(0.9, 0.85, 1.0));

// Fragment shader

@group(0) @binding(0)
var g_albedo: texture_2d<f32>;
@group(0) @binding(1)
var s_albedo: sampler;

@group(0) @binding(2)
var g_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@group(0) @binding(4)
var g_position: texture_2d<f32>;
@group(0) @binding(5)
var s_position: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_pos = textureSample(g_position, s_position, in.tex_coords).rgb;
    let normal = textureSample(g_normal, s_normal, in.tex_coords).rgb;
    let albedo = textureSample(g_albedo, s_albedo, in.tex_coords).rgb;

    var lighting = albedo * 0.01;

    let light_dir = normalize(light.position - frag_pos);
    let diffuse = max(dot(normal, light_dir), 0.0) * albedo * light.color;
    lighting += diffuse;

    return vec4<f32>(lighting, 1.0);
}
