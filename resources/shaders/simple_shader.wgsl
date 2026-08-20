struct VertexShaderOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f
}

@vertex
fn vs_main(
    @location(0) vertex_position: vec3f,
    @location(1) vertex_color: vec3f
) -> VertexShaderOutput {
    var output: VertexShaderOutput;
    output.position = vec4f(vertex_position, 1.0);
    output.color = vertex_color;
    return output;
}

@fragment
fn fs_main(input: VertexShaderOutput) -> @location(0) vec4f {
    return vec4f(input.color, 1.0);
}