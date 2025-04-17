use ui::{vulkan, shader};
shader!{terrain}
use vulkan::{Arc, Subbuffer, Image as GPUImage};
pub struct Terrain {
	pass: terrain::Pass,
	vertices: Subbuffer::<[terrain::Vertex]>,
	image: Arc<GPUImage>,
}

use {ui::Result, vector::{vec3, mat4}, image::{Image, rgb8, rgba8}};
use vulkan::{Context, from_iter, BufferUsage, Commands, ImageView, PrimitiveTopology, image, WriteDescriptorSet, linear};
impl Terrain {
	pub fn new(context: &Context, commands: &mut Commands, points: &[vec3], color: Image<&[rgb8]>) -> Result<Self> {
		let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, points.into_iter().map(|&v| terrain::Vertex{position: v.into()}))?;
		Ok(Self{
			pass: terrain::Pass::new(context, true, PrimitiveTopology::PointList)?,
			vertices,
			image: image(context, commands, color.map(|&v| rgba8::from(v)).as_ref())?
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertices, image} = self;
		pass.begin_rendering(context, commands, color, Some(depth), true, &terrain::Uniforms{
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