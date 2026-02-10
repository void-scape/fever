#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Fractal {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
	exponent: f32,
	burning_ship: u32,
	_pad: u32,
}

@group(2) @binding(0) var<uniform> args: Fractal;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

fn c_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn c_pow(z: vec2<f32>, n: f32) -> vec2<f32> {
    let r = length(z);
    if (r == 0.0) { return vec2(0.0); }
    let a = atan2(z.y, z.x);
    let rn = pow(r, n);
    let na = n * a;
    return vec2(rn * cos(na), rn * sin(na));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	let sz = textureDimensions(texture);
	let aspect = f32(sz.x) / f32(sz.y);
	let c = vec2(args.cx, args.cy);
	var z = (mesh.uv * 2.0 - 1.0) * args.zoom;
	for (var i = 0; i < i32(args.iterations); i++) {
		if length(z) > args.escape_radius * args.escape_radius {
			break;
		}
		if args.burning_ship == 1u {
			z = c_pow(vec2(abs(z.x), abs(z.y)), args.exponent) + c;
		} else {
			z = c_pow(z, args.exponent) + c;
		}
	}

	let fx = (z.x / aspect + 1.0) / 2.0;
	let fy = (z.y + 1.0) / 2.0;
	if fx >= 0.0 && fx < 1.0 && fy >= 0.0 && fy < 1.0 {
		return textureSample(texture, texture_sampler, vec2(fx, fy));
	} else {
	 	return vec4(0.0, 0.0, 0.0, 1.0);
	}
}
