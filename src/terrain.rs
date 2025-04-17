use ui::{vulkan, shader};
shader!{terrain} // z: f32, NdotL: f32
use vulkan::{Arc, Subbuffer, Image as GPUImage};
pub struct Terrain {
	pass: terrain::Pass,
	vertex_grid_size_x: u32,
	grid: Subbuffer::<[u32]>,
	vertices: Subbuffer::<[terrain::Vertex]>,
	image: Arc<GPUImage>,
}

use {ui::Result, vector::{xy, xyz, dot, normalize, cross, mat4}, image::Image};
use vulkan::{Context, buffer, BufferUsage, Commands, PrimitiveTopology, ImageView, WriteDescriptorSet, linear};
impl Terrain {
	pub fn new(context: &Context, _commands: &mut Commands, ground: &Image<impl AsRef<[f32]>>, meters_per_pixel: f32, z: impl Fn(f32)->f32, image: Arc<GPUImage>) -> Result<Self> {
		let ground = ground.as_ref();
		let size = ground.size;
		let vertex_grid_size_x = {assert_eq!(size.x, size.y); size.x};
		let vertex_stride = vertex_grid_size_x;
		let vertices = buffer(context, BufferUsage::VERTEX_BUFFER, ground.data.len())?;
		{
			let mut vertices = vertices.write()?;
			for y in 1..size.y-1 { for x in 1..size.x-1 {
				let dx_z = (ground[xy{x: x+1, y}]-ground[xy{x: x-1, y}])/(2.*meters_per_pixel);
				let dy_z = (ground[xy{x, y: y+1}]-ground[xy{x, y: y-1}])/(2.*meters_per_pixel);
				let n = normalize(cross(xyz{x: 1., y: 0., z: dx_z}, xyz{x: 0., y: 1., z: dy_z}));
				let NdotL = dot(n, xyz{x: 0., y: 0., z: 1.});
				assert!(NdotL >= 0. && NdotL <= 1.);
				vertices[((y-1)*vertex_stride+x-1) as usize] = terrain::Vertex{
					z: z(ground[xy{x,y}]),
					NdotL,
				};
			}}
		}
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, ((size.x-2)*(size.y-2)*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let mut target = 0;
			for y in 0..size.y-2 { for x in 0..size.x-2 {
				let i0 = y*vertex_stride+x;
				grid[target+0] = i0;
				grid[target+1] = i0+1;
				grid[target+2] = i0+vertex_stride+1;
				grid[target+3] = i0;
				grid[target+4] = i0+vertex_stride+1;
				grid[target+5] = i0+vertex_stride;
				target += 6;
			}}
			assert!(target == grid.len());
		}
		Ok(Self{
			pass: terrain::Pass::new(context, true, PrimitiveTopology::TriangleList)?,
			vertex_grid_size_x,
			grid,
			vertices,
			image
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
		let Self{pass, vertex_grid_size_x, grid, vertices, image} = self;
		pass.begin_rendering(context, commands, color, Some(depth), true, &terrain::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
			vertex_grid_size_x: *vertex_grid_size_x
		}, &[
			WriteDescriptorSet::image_view(1, ImageView::new_default(image.clone())?),
			WriteDescriptorSet::sampler(2, linear(context)),
		])?;
		commands.bind_index_buffer(grid.clone())?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw_indexed(grid.len() as _, 1, 0, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}