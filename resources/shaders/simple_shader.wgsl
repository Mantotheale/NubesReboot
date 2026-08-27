struct VertexShaderInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f
}

struct VertexShaderOutput {
    @builtin(position) position: vec4f,
    @location(0) tex_coords: vec2f
}

@vertex
fn vs_main(input: VertexShaderInput) -> VertexShaderOutput {
    var output: VertexShaderOutput;
    output.position = vec4f(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@group(0) @binding(0)
var tex_sampler: sampler;

@group(1) @binding(0)
var tex: texture_2d<f32>;

@fragment
fn fs_main(input: VertexShaderOutput) -> @location(0) vec4f {
    return textureSample(tex, tex_sampler, input.tex_coords);
}