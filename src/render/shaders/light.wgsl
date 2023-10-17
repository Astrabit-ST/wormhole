// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
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
    view_pos: vec4<f32>,
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
var g_color_roughness: texture_2d<f32>;
@group(2) @binding(1)
var s_color_roughness: sampler;

@group(2) @binding(2)
var g_normal_metallicity: texture_2d<f32>;
@group(2) @binding(3)
var s_normal_metallicity: sampler;

@group(2) @binding(4)
var g_position_occlusion: texture_2d<f32>;
@group(2) @binding(5)
var s_position_occlusion: sampler;

@group(2) @binding(6)
var g_emissive: texture_2d<f32>;
@group(2) @binding(7)
var s_emissive: sampler;

const PI = 3.14159265359;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    let color_roughness = textureSample(g_color_roughness, s_color_roughness, in.tex_coords);
    let normal_metallicity = textureSample(g_normal_metallicity, s_normal_metallicity, in.tex_coords);
    let position_occlusion = textureSample(g_position_occlusion, s_position_occlusion, in.tex_coords);
    let emissive = textureSample(g_emissive, s_emissive, in.tex_coords);

    let n = normalize(normal_metallicity.rgb);
    let v = normalize(camera.view_pos.rgb - position_occlusion.rgb);

    let f0 = mix(vec3<f32>(0.04), color_roughness.rgb, normal_metallicity.a);

    var l_o = vec3<f32>(0.0);
    for (var i = 0u; i < light_count; i++) {
        let light = lights[i];

        let l = normalize(light.position - position_occlusion.rgb);
        let h = normalize(v + l);

        let distance = length(light.position - position_occlusion.rgb);
        let attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

        let radiance = light.diffuse.rgb * attenuation;

        let ndf = distributionGGX(n, h, color_roughness.a);
        let g = geometrySmith(n, v, l, color_roughness.a);
        let f = fresnelSchlick(max(dot(h, v), 0.0), f0);

        let numerator = ndf * g * f;
        let denominator = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let k_s = f;
        let k_d = (vec3(1.0) - k_s) * 1.0 - normal_metallicity.a;

        let n_dot_l = max(dot(n, l), 0.0);
        l_o += (k_d * color_roughness.rgb / PI + specular) * radiance * n_dot_l;
    }
    let ambient = vec3<f32>(0.03) * color_roughness.rgb * position_occlusion.a;

    var color = ambient + l_o + emissive.rgb;
    // color = color / (color + vec3(1.0));
    // color = pow(color, vec3(1.0 / 2.2));

    out.color = vec4<f32>(color, 1.0);

    return out;
}

fn fresnelSchlick(cosTheta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}


fn distributionGGX(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h = max(dot(n, h), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;

    let num = a2;
    var denom = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn geometrySchlickGGX(n_dot_v: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;

    return num / denom;
}

fn geometrySmith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let ggx2 = geometrySchlickGGX(n_dot_v, roughness);
    let ggx1 = geometrySchlickGGX(n_dot_l, roughness);

    return ggx1 * ggx2;
}