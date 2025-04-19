@group(0) @binding(0) var<uniform> matrix: mat4x4<f32>;

@group(1) @binding(0) var texture_image: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const POS = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0)
);

@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {

    let coord = POS[index];
    let clip = vec4<f32>(coord, 0.0, 1.0);

    return VertexOutput(
        clip,
        coord
    );
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_image, texture_sampler, in.uv);
}


