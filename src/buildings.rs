use ui::{vulkan, shader};
shader!{buildings}
use vulkan::{Arc, Subbuffer, Image as GPUImage};
pub struct Buildings {
	pass: buildings::Pass,
	vertices: Subbuffer::<[buildings::Vertex]>,
	indices: Subbuffer::<[u32]>,
	image: Arc<GPUImage>,
}

use {ui::Result, vector::{vec3, mat4}};
use vulkan::{Context, Commands, from_iter, BufferUsage, buffer, ImageView, PrimitiveTopology, WriteDescriptorSet, linear};
impl Buildings {
	pub fn new(context: &Context, _commands: &mut Commands, quads: &[vec3], map: impl Fn(vec3)->vec3, image: Arc<GPUImage>) -> Result<Self> {
		let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, quads.into_iter().map(|&p| buildings::Vertex{position: map(p).into()}))?;
		let indices = buffer(context, BufferUsage::INDEX_BUFFER, quads.len()*6)?;
		{
			let mut indices = indices.write()?;
			for i in 0..quads.len()/4 {
				indices[i*6+0] = (i*4+0) as u32;
				indices[i*6+1] = (i*4+2) as u32;
				indices[i*6+2] = (i*4+1) as u32;
				indices[i*6+3] = (i*4+0) as u32;
				indices[i*6+4] = (i*4+3) as u32;
				indices[i*6+5] = (i*4+2) as u32;
			}
		}
		Ok(Self{
			pass: buildings::Pass::new(context, true, PrimitiveTopology::TriangleList)?,
			vertices,
			indices,
			image
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertices, indices, image} = self;
		pass.begin_rendering(context, commands, color, Some(depth), false, &buildings::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
		}, &[
			WriteDescriptorSet::image_view(1, ImageView::new_default(image.clone())?),
			WriteDescriptorSet::sampler(2, linear(context)),
		])?;
		commands.bind_index_buffer(indices.clone())?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw_indexed(indices.len() as _, 1, 0, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}
