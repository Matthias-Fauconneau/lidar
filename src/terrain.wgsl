struct Uniforms { view_projection: mat4x4<f32> }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Vertex {
	@location(0) position: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
}

@vertex fn vertex(vertex: Vertex) -> VertexOutput {
	let position = uniforms.view_projection * vec4(vertex.position, 1.);
	return VertexOutput(position);
}

@fragment fn fragment(vertex: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(1.,1.,1.,1.);
}
