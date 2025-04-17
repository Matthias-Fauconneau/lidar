use ui::{vulkan, shader};
shader!{points}
use vulkan::{Arc, Subbuffer, Image as GPUImage};
pub struct Points {
	pass: points::Pass,
	vertices: Subbuffer::<[points::Vertex]>,
	image: Arc<GPUImage>,
}

use {ui::Result, vector::{xyz, vec3, mat4}};
use vulkan::{Context, from_iter, BufferUsage, Commands, ImageView, PrimitiveTopology, WriteDescriptorSet, linear};
impl Points {
	pub fn new(context: &Context, _commands: &mut Commands, points: &[vec3], map: impl Fn(f32)->f32, image: Arc<GPUImage>) -> Result<Self> {
		let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, points.into_iter().map(|&xyz{x,y,z}| points::Vertex{position: xyz{x,y,z:map(z)}.into()}))?;
		Ok(Self{
			pass: points::Pass::new(context, true, PrimitiveTopology::PointList)?,
			vertices,
			image
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertices, image} = self;
		pass.begin_rendering(context, commands, color, Some(depth), false, &points::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
		}, &[
			WriteDescriptorSet::image_view(1, ImageView::new_default(image.clone())?),
			WriteDescriptorSet::sampler(2, linear(context)),
		])?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw(vertices.len() as _, 1, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}