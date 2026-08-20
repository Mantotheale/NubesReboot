@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> @builtin(position) vec4f {
    const vertices = array<vec3f, 3>(
        vec3f(-0.5, -0.5, 0),
        vec3f(0.5, -0.5, 0),
        vec3f(0, 0.5, 0),
    );

    return vec4f(vertices[vertex_index], 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(0.1, 0.2, 0.3, 1.0);
}