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

// Fragment shader
struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

struct Camera {
    viewport_size: vec2<f32>,
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,

    position: vec3<f32>,
}
@group(1) @binding(0)
var<storage> lights: array<Light>;

var<push_constant> light_count: u32;

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

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    var result = vec3<f32>();
    for (var i = 0u; i < light_count; i++) {
        let light = lights[i];

        let frag_pos = textureSample(g_position, s_position, in.tex_coords).rgb;
        let normal = textureSample(g_normal, s_normal, in.tex_coords).rgb;
        let albedo = textureSample(g_albedo, s_albedo, in.tex_coords).rgb;

        let ambient = light.ambient.rgb * albedo;

        let light_dir = normalize(light.position - frag_pos);
        let diffuse_strength = max(dot(normal, light_dir), 0.0);
        let diffuse = light.diffuse.rgb * diffuse_strength * albedo;

        let view_dir = normalize(camera.view_pos.xyz - frag_pos);
        let half_dir = normalize(view_dir + light_dir);
        let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
        let specular = light.specular.rgb * specular_strength;

        let distance = length(light.position - frag_pos);
        let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

        let ambient_color = ambient * attenuation;
        let diffuse_color = diffuse * attenuation;
        let specular_color = specular * attenuation;

        result += ambient_color + diffuse_color + specular_color;
    }

    out.color = vec4<f32>(result, 1.0);

    return out;
}
