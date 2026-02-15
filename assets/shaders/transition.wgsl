// wgsl port of this shader: https://godotshaders.com/shader/fractal-noise-scene-transition/

// Pixelated noise transition effect based on https://godotshaders.com/shader/warped-fractal-noise/
//
// Copyright Gerardo Montaño 2025
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of 
// this software and associated documentation files (the “Software”), to deal in 
// the Software without restriction, including without limitation the rights to 
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies 
// of the Software, and to permit persons to whom the Software is furnished to do 
// so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all 
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR 
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, 
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL 
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER 
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, 
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE 
// SOFTWARE.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct TransitionUniform {
	color_low: vec4<f32>,
	color_mid: vec4<f32>,
	color_high: vec4<f32>,
    progress: f32,
    speed: f32,
    zoom: f32,
    background_threshold: f32,
    color_low_threshold: f32,
    color_mid_threshold: f32,
    seed: f32,
	time: f32,
};

@group(0) @binding(2) var<uniform> args: TransitionUniform;

fn rand(n: vec2<f32>) -> f32 {
    return fract(sin(dot(n, vec2(12.9898 + args.seed, 4.1414 - args.seed))) * (43758.5453 + args.seed * 1000.0));
}

fn noise(p: vec2<f32>) -> f32 {
    let ip = floor(p);
    var u = fract(p);
    u = u * u * (3.0 - 2.0 * u);
    let res = mix(
        mix(rand(ip), rand(ip + vec2(1.0, 0.0)), u.x),
        mix(rand(ip + vec2(0.0, 1.0)), rand(ip + vec2(1.0, 1.0)), u.x), 
        u.y
    );
    return res * res;
}

fn fbm(in: vec2<f32>) -> f32 {
    var p = in;
    var f = 0.0;
    let time = args.time * args.speed - args.seed;
    let mtx = mat2x2f(vec2(0.80, -0.60), vec2(0.60, 0.80));
    f += 0.500000 * noise(p + vec2(time)); 
    p = mtx * p * 2.02;
    f += 0.031250 * noise(p); 
    p = mtx * p * 2.01;
    f += 0.250000 * noise(p); 
    p = mtx * p * 2.03;
    f += 0.125000 * noise(p); 
    p = mtx * p * 2.01;
    f += 0.062500 * noise(p); 
    p = mtx * p * 2.04;
    f += 0.015625 * noise(p + vec2(sin(time)));
    return f / 0.96875;
}

fn pattern(p: vec2<f32>) -> f32 {
    return fbm(p + vec2(fbm(p + vec2(fbm(p)))));
}

fn colormap(in: f32, uv: vec2<f32>) -> vec4<f32> {
    var x = in;
    let sz = textureDimensions(screen_texture);
    let aspect = f32(sz.x) / f32(sz.y);
    let sweep = (uv.x * aspect + uv.y) / (aspect + 1.0);
    let val = max(0.0, min(-abs(args.progress * 4.0 - sweep * 2.0 - 1.0) + 1.0, 1.0) * 2.0);
    x *= val;
    if x < args.background_threshold { 
        return vec4(0.0, 0.0, 0.0, 0.0);
	} else if x < args.color_low_threshold { 
        return mix(
			vec4(0.0, 0.0, 0.0, 0.0),
			args.color_low,
			(x - args.background_threshold) / (args.color_low_threshold - args.background_threshold)
		);
    } else if x < args.color_mid_threshold { 
        return mix(
			args.color_low,
			args.color_mid,
			(x - args.color_low_threshold) / (args.color_mid_threshold - args.color_low_threshold)
		);
    } else {
		return mix(
			args.color_mid,
			args.color_high,
			(x - args.color_mid_threshold) / (1.0 - args.color_mid_threshold)
		);
	}
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let sz = textureDimensions(screen_texture);
    let aspect = f32(sz.x) / f32(sz.y);
    let modifier = 1.0 / (1.0 / vec2(f32(sz.x), f32(sz.y)));
    let grid_uv = floor(in.uv * modifier) / modifier;
    let aspect_uv = vec2(grid_uv.x * aspect, grid_uv.y);
    let shade = pattern(aspect_uv * args.zoom);
    let noise_color = colormap(shade, grid_uv);
    let screen = textureSample(screen_texture, screen_sampler, in.uv);
    return mix(screen, noise_color, noise_color.a);
}
