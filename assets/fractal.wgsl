#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Fractal {
    escape_radius: f32,
    iterations: f32,
    cx: f32,
    cy: f32,
    zoom: f32,
}

@group(2) @binding(0) var<uniform> args: Fractal;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
	let sz = textureDimensions(texture);
	let aspect = f32(sz.x) / f32(sz.y);
	var z = (mesh.uv * 2.0 - 1.0) * args.zoom;

	let rotation = 0.01;
	let cos_r = cos(rotation);
	let sin_r = sin(rotation);

	for (var i = 0; i < i32(args.iterations); i++) {
		if length(z) > args.escape_radius * args.escape_radius {
			break;
		}

		let rotated_x = z.x * cos_r - z.y * sin_r;
		let rotated_y = z.x * sin_r + z.y * cos_r;
		z = vec2(rotated_x, rotated_y);

		let zxt = z.x * z.x - z.y * z.y + args.cx;
		z.y = 2.0 * z.x * z.y + args.cy;
		z.x = zxt;
	}

	let fx = (z.x / aspect + 1.0) / 2.0;
	let fy = (z.y + 1.0) / 2.0;
	if fx >= 0.0 && fx < 1.0 && fy >= 0.0 && fy < 1.0 {
		return textureSample(texture, texture_sampler, vec2(fx, fy));
	} else {
	 	return vec4(0.0, 0.0, 0.0, 1.0);
	}
}
