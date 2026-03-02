// Vertex shader: renders textured quads with per-instance transform.
// Each quad has a transform uniform controlling position and scale in NDC space,
// plus a tint vec4 for alpha-only texture support.

struct QuadUniforms {
    // xy = position offset (NDC), zw = scale
    transform: vec4<f32>,
    // rgb = tint color, a = 1.0 means alpha-only texture (R8Unorm)
    tint: vec4<f32>,
};

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@group(1) @binding(0)
var<uniform> quad: QuadUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // 6 vertices for a quad (2 triangles)
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );

    var output: VertexOutput;
    let pos = positions[vertex_index];
    // Apply scale then offset
    output.position = vec4<f32>(pos * quad.transform.zw + quad.transform.xy, 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(t_texture, t_sampler, input.uv);
    if (quad.tint.a > 0.5) {
        // Alpha-only texture (BC4/R8Unorm): .r channel holds alpha value
        let a = sample.r;
        return vec4<f32>(quad.tint.rgb * a, a); // premultiplied output
    } else {
        // RGBA texture (BC7/Rgba8Unorm): premultiplied, apply tint
        return vec4<f32>(sample.rgb * quad.tint.rgb, sample.a);
    }
}
