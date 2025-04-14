//use ui::{vulkan, shader};
//shader!{terrain} // z: f32, NdotL: f32, water: f32,
use vector::vec3;
struct Vertex {
	position: vec3/*<f32>*/,
}
//use vulkan::{Arc, Subbuffer, Image as GPUImage};
pub struct Terrain {
	//pass: terrain::Pass,
	/*vertex_grid_size_x: u32,
	grid: Subbuffer::<[u32]>,
	vertices: Subbuffer::<[terrain::Vertex]>,
	image: Arc<GPUImage>,*/
}

/*use {ui::Result, std::cmp::min, num::sq, vector::{xy, xyz, dot, normalize, cross, mat4}, image::{Image, rgba8}};
use vulkan::{Context, buffer, BufferUsage, Commands, ImageView, image, WriteDescriptorSet, linear};*/
impl Terrain {
	/*pub fn new(context: &Context, commands: &mut Commands, ground: &Image<impl AsRef<[f32]>>, water: &Image<impl AsRef<[f32]>>, meters_per_pixel: f32, z: impl Fn(f32)->f32, color: &Image<impl AsRef<[rgba8]>>) -> Result<Self> {
		let [ground, water] = [ground.as_ref(), water.as_ref()];
		let size = ground.size;
		let vertex_grid_size_x = {assert_eq!(size.x, size.y); size.x};
		let vertex_stride = vertex_grid_size_x;
		let vertices = buffer(context, BufferUsage::VERTEX_BUFFER, ground.data.len())?;
		{
			let mut vertices = vertices.write()?;
			for y in 1..size.y-1 { for x in 1..size.x-1 {
				let dx_z = (water[xy{x: x+1, y}]-water[xy{x: x-1, y}])/(2.*meters_per_pixel);
				let dy_z = (water[xy{x, y: y+1}]-water[xy{x, y: y-1}])/(2.*meters_per_pixel);
				let n = normalize(cross(xyz{x: 1., y: 0., z: dx_z}, xyz{x: 0., y: 1., z: dy_z}));
				let NdotL = dot(n, xyz{x: 0., y: 0., z: 1.});
				assert!(NdotL >= 0. && NdotL <= 1.);
				vertices[(y*vertex_stride+x) as usize] = terrain::Vertex{
					z: z(water[xy{x,y}]),
					NdotL,
					water: if water[xy{x,y}] > ground[xy{x,y}] { 1. } else { 0. } // Single terrain for water/ground (`water` has ground altitude for points without water)
				};
			}}
		}
		let mut cell_count = 0;
		let center = size.signed()/2;
		let radius2 = sq(min(size.x, size.y)/2);
		for y in 0..size.y-2 { for x in 0..size.x-2 {
			let r2 = vector::sq(xy{x,y}.signed()-center) as u32;
			if r2 >= radius2 { continue; }
			let i0 = y*vertex_stride+x;
			if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| ground[(i0+di) as usize] > f32::MIN) {} else {continue;}
			cell_count += 1;
		}}
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, (cell_count*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let mut target = 0;
			for y in 0..size.y-2 { for x in 0..size.x-2 {
				let r2 = vector::sq(xy{x,y}.signed()-center) as u32;
				if r2 >= radius2 { continue; }
				let i0 = y*vertex_stride+x;
				if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| ground[(i0+di) as usize] > f32::MIN) {} else {continue;}
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
			pass: terrain::Pass::new(context, true)?,
			vertex_grid_size_x,
			grid,
			vertices,
			image: image(context, commands, color.as_ref())?
		})
	}*/
	/*pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) -> Result {
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
	}*/
}