use ui::{vulkan, shader};
shader!{terrain}
use vulkan::Subbuffer;
pub struct Terrain {
	pass: terrain::Pass,
	vertices: Subbuffer::<[terrain::Vertex]>,
}

use {ui::Result, vector::{vec3, mat4}};
use vulkan::{Context, from_iter, BufferUsage, Commands, Arc, ImageView, PrimitiveTopology};
impl Terrain {
	pub fn new(context: &Context, _: &mut Commands, points: &[vec3]) -> Result<Self> {
		let vertices = from_iter(context, BufferUsage::VERTEX_BUFFER, points.into_iter().map(|&v| terrain::Vertex{position: v.into()}))?;
		Ok(Self{
			pass: terrain::Pass::new(context, true, PrimitiveTopology::PointList)?,
			vertices,
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertices} = self;
		pass.begin_rendering(context, commands, color, Some(depth), true, &terrain::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
		}, &[])?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw(vertices.len() as _, 1, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}