struct Uniforms { view_projection: mat4x4<f32>, vertex_grid_size_x: u32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var linear: sampler;

struct Vertex {
	@location(0) z: f32,
	@location(1) NdotL: f32,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) texture_coordinates: vec2<f32>,
	@location(1) NdotL: f32,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, vertex: Vertex) -> VertexOutput {
	let xy_01 = vec2(f32(vertex_index % uniforms.vertex_grid_size_x), f32(vertex_index / uniforms.vertex_grid_size_x)) / f32(uniforms.vertex_grid_size_x);
	let xy = xy_01 * 2. - 1.;
	let position = uniforms.view_projection * vec4(xy, vertex.z, 1.);
	return VertexOutput(position, xy_01, vertex.NdotL);
}

@fragment fn fragment(vertex: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(image, linear, vertex.texture_coordinates)*vertex.NdotL;
}
