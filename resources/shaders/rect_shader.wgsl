struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tex_idx: i32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) @interpolate(flat) tex_idx: i32
};

@group(0) @binding(0)
var tex0: texture_2d<f32>;
@group(0) @binding(1)
var tex1: texture_2d<f32>;
@group(0) @binding(2)
var tex2: texture_2d<f32>;
@group(0) @binding(3)
var tex3: texture_2d<f32>;
@group(0) @binding(4)
var tex4: texture_2d<f32>;
@group(0) @binding(5)
var tex5: texture_2d<f32>;
@group(0) @binding(6)
var tex6: texture_2d<f32>;
@group(0) @binding(7)
var tex7: texture_2d<f32>;
@group(0) @binding(8)
var tex8: texture_2d<f32>;
@group(0) @binding(9)
var tex9: texture_2d<f32>;
@group(0) @binding(10)
var tex10: texture_2d<f32>;
@group(0) @binding(11)
var tex11: texture_2d<f32>;
@group(0) @binding(12)
var tex12: texture_2d<f32>;
@group(0) @binding(13)
var tex13: texture_2d<f32>;
@group(0) @binding(14)
var tex14: texture_2d<f32>;
@group(0) @binding(15)
var tex15: texture_2d<f32>;
@group(0) @binding(16)
var tex_sampler: sampler;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    out.tex_coords = in.tex_coords;
    out.tex_idx = in.tex_idx;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    switch (in.tex_idx) {
        case 0: {
            return textureSample(tex0, tex_sampler, in.tex_coords);
        }
        case 1: {
            return textureSample(tex1, tex_sampler, in.tex_coords);
        }
        case 2: {
            return textureSample(tex2, tex_sampler, in.tex_coords);
        }
        case 3: {
            return textureSample(tex3, tex_sampler, in.tex_coords);
        }
        case 4: {
            return textureSample(tex4, tex_sampler, in.tex_coords);
        }
        case 5: {
            return textureSample(tex5, tex_sampler, in.tex_coords);
        }
        case 6: {
            return textureSample(tex6, tex_sampler, in.tex_coords);
        }
        case 7: {
            return textureSample(tex7, tex_sampler, in.tex_coords);
        }
        case 8: {
            return textureSample(tex8, tex_sampler, in.tex_coords);
        }
        case 9: {
            return textureSample(tex9, tex_sampler, in.tex_coords);
        }
        case 10: {
            return textureSample(tex10, tex_sampler, in.tex_coords);
        }
        case 11: {
            return textureSample(tex11, tex_sampler, in.tex_coords);
        }
        case 12: {
            return textureSample(tex12, tex_sampler, in.tex_coords);
        }
        case 13: {
            return textureSample(tex13, tex_sampler, in.tex_coords);
        }
        case 14: {
            return textureSample(tex14, tex_sampler, in.tex_coords);
        }
        case 15: {
            return textureSample(tex15, tex_sampler, in.tex_coords);
        }
        default: {
            return in.color;
        }
    }
}