struct Uniforms { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var image: texture_2d<f32>;
@group(0) @binding(2) var linear: sampler;

struct Vertex {
	@location(0) position: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) texture_coordinates: vec2<f32>,
}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
	let position = uniforms.view_projection * vec4(vertex.position, 1.);
	let texture_coordinates = (vertex.position.xy+1.)/2.;
	return VertexOutput(position, texture_coordinates);
}

@fragment fn fragment(vertex: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(image, linear, vertex.texture_coordinates);
}
