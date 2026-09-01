struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) pixel_width: f32,
    @location(3) tex_coords: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) half_width_px: f32,
    @location(2) distance_vector: vec2<f32>
};

struct ScreenDimensions {
    width: u32,
    height: u32,
};

@group(0) @binding(0)
var<uniform> screen_dimensions: ScreenDimensions;
@group(1) @binding(0)
var tex_sampler: sampler;
@group(1) @binding(1)
var tex: texture_2d<f32>;

const FRINGE: f32 = 1.0;
const HALF_FRINGE: f32 = FRINGE * 0.5;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let half_width_px = in.pixel_width * 0.5;

    let distance_vector = in.normal * (half_width_px + HALF_FRINGE);
    let ndc_offset = distance_vector * vec2<f32>(
        2.0 / f32(screen_dimensions.width),
        2.0 / f32(screen_dimensions.height)
    );

    let offset_position = in.position + ndc_offset;

    out.clip_position = vec4<f32>(offset_position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.half_width_px = half_width_px;
    out.distance_vector = distance_vector;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let inner = in.half_width_px - HALF_FRINGE;
    let outer = in.half_width_px + HALF_FRINGE;
    let squared_distance = dot(in.distance_vector, in.distance_vector);

    let alpha = 1 - smoothstep(inner * inner, outer * outer, squared_distance);
    let base_color = textureSample(tex, tex_sampler, in.tex_coords);
    return vec4<f32>(base_color.rgb, base_color.a * alpha);
}