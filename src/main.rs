#![feature(slice_from_ptr_range)] // shader
#![allow(non_snake_case)] 
use ui::{Error, Result, throws};

use owning_ref::OwningRef;
type Array<T=f32> = OwningRef<Box<memmap::Mmap>, [T]>;
#[throws] fn map<T:bytemuck::Pod>(path: &str) -> Array<T> {
    OwningRef::new(Box::new(unsafe{memmap::Mmap::map(&std::fs::File::open(path)?)}?)).map(|data| bytemuck::cast_slice(&*data))
}

mod terrain; use terrain::Terrain;

use {vector::{xyz, MinMax, vec3, xy, size, int2, vec2, rotate, xyzw, vec4, mat4}, image::{Image, rgb, rgba, rgba8, rgb8}};
use ui::vulkan::{Context, Commands, Arc, ImageView, Image as GPUImage, default, ImageCreateInfo, Format, ImageUsage};

struct App {
	terrain: Terrain,
	view_position: vec2,
	yaw: f32,
}

impl App {
fn new(context: &Context, commands: &mut Commands) -> Result<Self> {
	let name = "2684000_1248500";
	let ref cache = format!("{}/.cache/lidar/{name}", std::env::var("HOME")?);
	if !std::fs::exists(cache)? {
		let mut reader = las::Reader::from_path(format!("{}/{name}.laz", std::env::var("HOME")?))?;
		let mut points = Vec::with_capacity(reader.header().number_of_points() as usize);
		let las::Bounds{min, max} = reader.header().bounds();
		let MinMax{min, max} = MinMax{min: {let las::Vector{x,y,z}=min; xyz{x,y,z}}, max: {let las::Vector{x,y,z}=max; xyz{x,y,z}}};
		//println!("{min:?} {max:?}");
		let center = (1./2.)*(min+max);
	    let extent = max-min;
		println!("{center:?} {extent:?}");
	    let extent = extent.x.min(extent.y);
	    
		for point in reader.points() {
			let las::Point{x: E,y: N, z, ..} = point.unwrap();
			let x = 2.*(E-center.x)/extent;
	        let y = 2.*(N-center.y)/extent;
	        let z = 2.*(z-center.z)/extent;
			points.push(xyz{x,y,z}.into());
		}
		std::fs::write(cache, bytemuck::cast_slice::<vec3,_>(&points))?;
		println!("{cache}");
	}
	let ref points = map::<vec3>(cache)?;
	let points_bounds = MinMax{min: xy{x: 2684000, y: 1248500}, max: xy{ x: 2684500, y: 1249000}};
	println!("{points_bounds:?}");
	//let [center, extent] = [xy{x: 2684250, y: 1248750}, xy{ x: 500, y: 500}];
	
	let name = "2408";
	let ref cache = format!("{}/.cache/lidar/{name}", std::env::var("HOME")?);
	if !std::fs::exists(cache)? {
		let tiff = unsafe{memmap::Mmap::map(&std::fs::File::open(format!("{}/{name}.tif", std::env::var("HOME")?))?)?};
	    let mut tiff = tiff::decoder::Decoder::new(std::io::Cursor::new(&*tiff))?;
	    let size = {let (x, y) = tiff.dimensions()?; xy{x, y}};
		let min = {let [_, _, _, E, N, _] = tiff.get_tag_f64_vec(tiff::tags::Tag::ModelTiepointTag)?[..] else { panic!() }; xy{x: E as u32, y: N as u32}};
		let scale = {let [scale_E, scale_N, _] = tiff.get_tag_f64_vec(tiff::tags::Tag::ModelPixelScaleTag)?[..] else { panic!() }; xy{x: (1./scale_E) as u32, y: (1./scale_N) as u32}};
		let MinMax{min, max} = MinMax{min: (points_bounds.min-min)*scale+xy{x: 0, y: size.y}, max: (points_bounds.max-min)*scale+xy{x: 0, y: size.y}};
	    let mut image = Image::zero(max-min);
		let tile_size = {let (x,y) = tiff.chunk_dimensions(); xy{x,y}};
		let tiles_stride = (size.x+tile_size.x-1) / tile_size.x;
		let tiles = MinMax{min: min/tile_size, max: max/tile_size};
		for y0 in tiles.min.y..tiles.max.y { for x0 in tiles.min.x..tiles.max.x {
			let i = y0*tiles_stride+x0;
			let tiff::decoder::DecodingResult::U8(data) = tiff.read_chunk(i)? else {unimplemented!()};
			let tile = Image::new(tiff.chunk_data_dimensions(i).into(), bytemuck::cast_vec::<_, rgba8>(data));
			for y in 0..tile.size.y { for x in 0..tile.size.x {
				if let Some(p) = image.get_mut(xy{x: x0*tile_size.x+x, y: y0*tile_size.y+y}) { *p = {let rgba{r,g,b,a:_} = tile[xy{x,y}]; rgb{r,g,b}}; } // FIXME
			}}
		}}
		std::fs::write(cache, bytemuck::cast_slice(&image.data))?;
		println!("{cache}");
	}
	let image = map::<rgb8>(cache)?;
	let image = Image::new(10000.into(), image);
	Ok(Self{
		terrain: Terrain::new(context, commands, points, image.as_ref())?,
		view_position: xy{x: 0., y: 0.}, yaw: 0.
	})
}}

impl ui::Widget for App {
fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
	let Self{terrain, view_position, yaw} = self;
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
	let aspect_ratio = image_size.x as f32/image_size.y as f32;
	
	let view_projection = |xyz{x,y,z}:vec3| {
		let xy{x,y} = rotate(*yaw, xy{x,y} - *view_position);
		let xy{x: y, y: z} = rotate(-std::f32::consts::PI/3., xy{x: y, y: z});
		let z = (z-1.)/2.;
		let n = 1./4.;
		let f = 2.;
		if false { xyzw{x, y: aspect_ratio*y, z: -z, w: 1.}  } else {
			let zz = -f/(f-n);
			let z1 = -(f*n)/(f-n);
			xyzw{x, y: aspect_ratio*y, z: zz*z+z1, w: -z}
		}
	};
	fn from_linear(linear: impl Fn(vec3)->vec4) -> mat4 {
		let w = linear(xyz{x:0.,y:0.,z:0.});
		let xyz{x,y,z} = xyz{x: xyz{x:1.,y:0.,z:0.}, y:xyz{x:0.,y:1.,z:0.}, z: xyz{x:0.,y:0.,z:1.}}.map(|e| linear(e)-w);
		xyzw{x,y,z,w}
	}
	let view_projection = from_linear(view_projection);
	
	let depth = ImageView::new_default(GPUImage::new(memory_allocator.clone(), ImageCreateInfo{
	format: Format::D16_UNORM,
	extent: target.image().extent(),
	usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
	..default()
	}, default())?)?;
	
	terrain.render(context, commands, target.clone(), depth.clone(), view_projection)?;
	
	*yaw += std::f32::consts::PI/6./60.;
	Ok(())
}
fn event(&mut self, _context: &Context, _commands: &mut Commands, _size: size, _event_context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}

fn main() -> Result { ui::run("lidar", Box::new(move |context, commands| Ok(Box::new(App::new(context, commands)?)))) }

