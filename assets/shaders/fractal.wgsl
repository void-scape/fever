#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Fractal {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
	exponent: f32,
	burning_ship: u32,
	mandelbrot: u32,
	opacity: f32,
	rotation: f32,
	_pad: vec2<f32>,
}

@group(2) @binding(0) var<uniform> args: Fractal;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn cpow(z: vec2<f32>, n: f32) -> vec2<f32> {
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

	var c: vec2<f32>;
	var z: vec2<f32>;
	let p = (mesh.uv * 2.0 - 1.0) * args.zoom;

    let rot = vec2(cos(args.rotation), sin(args.rotation));
    let point = cmul(p, rot);

	if args.mandelbrot == 1 {
		c = point + vec2(args.cx, args.cy);
		z = vec2(0.0, 0.0);
	} else {
		c = vec2(args.cx, args.cy);
		z = point;
	}

	for (var i = 0; i < i32(args.iterations); i++) {
		if dot(z, z) > args.escape_radius * args.escape_radius {
			break;
		}
		if args.burning_ship == 1u {
			z = cpow(vec2(abs(z.x), abs(z.y)), args.exponent) + c;
		} else {
			z = cpow(z, args.exponent) + c;
		}
	}

	var nz = z;
	if fract(args.iterations) != 0.0 {
		if dot(z, z) <= args.escape_radius * args.escape_radius {
			if args.burning_ship == 1u {
				nz = cpow(vec2(abs(z.x), abs(z.y)), args.exponent) + c;
			} else {
				nz = cpow(z, args.exponent) + c;
			}
		}
	}

	let fz = mix(z, nz, fract(args.iterations));
	let fx = (fz.x / aspect + 1.0) / 2.0;
	let fy = (fz.y + 1.0) / 2.0;
	if fx >= 0.0 && fx < 1.0 && fy >= 0.0 && fy < 1.0 {
		return vec4(textureSample(texture, texture_sampler, vec2(fx, fy)).rgb, args.opacity);
	} else {
	 	return vec4(0.0, 0.0, 0.0, args.opacity);
	}
}
