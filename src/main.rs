#![feature(slice_from_ptr_range)] // shader
#![allow(non_snake_case)] 
use ui::{Error, Result, throws};

use owning_ref::OwningRef;
type Array<T=f32> = OwningRef<Box<memmap::Mmap>, [T]>;
#[throws] fn map<T:bytemuck::Pod>(path: &str) -> Array<T> {
	OwningRef::new(Box::new(unsafe{memmap::Mmap::map(&std::fs::File::open(path)?)}?)).map(|data| bytemuck::cast_slice(&*data))
}

mod terrain; use terrain::Terrain;
mod points; use points::Points;

use {vector::{xyz, MinMax, vec3, xy, size, int2, vec2, rotate, xyzw, vec4, mat4}, image::{Image, rgb8, rgba8}};
use ui::vulkan::{Context, Commands, Arc, ImageView, Image as GPUImage, default, ImageCreateInfo, Format, ImageUsage, image};

struct App {
	terrain: Terrain,
	points: Points,
	view_position: vec2,
	yaw: f32,
}

impl App {
	fn new(context: &Context, commands: &mut Commands) -> Result<Self> {
		let name = "2684000_1248500";
		let ref cache = format!("{}/.cache/lidar/{name}.z", std::env::var("HOME")?);
		if !std::fs::exists(cache)? {
			let file = std::fs::File::open(format!("{}/{name}.tif", std::env::var("HOME")?))?;
			let mut tiff = tiff::decoder::Decoder::new(file)?;
			let tiff::decoder::DecodingResult::F32(data) = tiff::decoder::Decoder::read_image(&mut tiff)? else {unimplemented!()};
			let size = {let (x, y) = tiff.dimensions()?; xy{x, y}};
			let mut ground = Image::uninitialized(size);
			for y in 0..size.y { 
				let source = (y*size.x) as usize; 
				let target = ((size.y-1-y)*size.x) as usize; 
				ground.data[target..target+size.x as usize].copy_from_slice(&data[source..source+size.x as usize]); 
			}
			std::fs::write(cache, bytemuck::cast_slice::<f32,_>(&ground.data))?;
		}
		let ground = map::<f32>(cache)?;
		let ground = Image::new(2000.into(), ground);
		let meters_per_pixel = 1000./ground.size.x as f32;
		
		let ref cache = format!("{}/.cache/lidar/{name}", std::env::var("HOME")?);
		if !std::fs::exists(cache)? {
			let mut reader = las::Reader::from_path(format!("{}/{name}.laz", std::env::var("HOME")?))?;
			let mut points = Vec::with_capacity(reader.header().number_of_points() as usize);
			let las::Bounds{min, max} = reader.header().bounds();
			let MinMax{min, max} = MinMax{min: {let las::Vector{x,y,z}=min; xyz{x,y,z}}, max: {let las::Vector{x,y,z}=max; xyz{x,y,z}}};
			let center = (1./2.)*(min+max);
			let extent = max-min;
			let extent = extent.x.min(extent.y);

			for point in reader.points() {
				let las::Point{x: E,y: N, z, ..} = point.unwrap();
				let x = 2.*(E-center.x)/extent;
				let y = 2.*(N-center.y)/extent;
				//let z = 2.*(z-center.z)/extent;
				points.push(xyz{x,y,z}.into());
			}
			std::fs::write(cache, bytemuck::cast_slice::<vec3,_>(&points))?;
			println!("{cache}");
		}
		let ref points = map::<vec3>(cache)?;
		let points_bounds = MinMax{min: xy{x: 2684000, y: 1248500}, max: xy{ x: 2684500, y: 1249000}};

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
			for y0 in tiles.min.y..=tiles.max.y { for x0 in tiles.min.x..=tiles.max.x {
				let chunk = (y0*tiles_stride+x0) as usize;
				std::io::Seek::seek(&mut tiff.reader, std::io::SeekFrom::Start(tiff.image.chunk_offsets[chunk]))?;
				let chunk_len = tiff.image.chunk_bytes[chunk as usize];
				let jpeg_tables_len = tiff.image.jpeg_tables.as_ref().map(|jpeg_tables| jpeg_tables.len()-2).unwrap_or(0);
				let mut buffer = vec![0; jpeg_tables_len+chunk_len as usize];
				use std::io::Read;
				if let Some(jpeg_tables) = &tiff.image.jpeg_tables {
					buffer[..jpeg_tables_len].copy_from_slice(&jpeg_tables[..jpeg_tables_len]);
					tiff.reader.read_exact(&mut [0; 2])?;
				}
				tiff.reader.read_exact(&mut buffer[jpeg_tables_len..])?;
				let mut jpeg = jpeg::JpegDecoder::new_with_options(buffer, 
					jpeg::zune_core::options::DecoderOptions::new_safe().jpeg_set_out_colorspace(jpeg::zune_core::colorspace::ColorSpace::RGB)
				);
				jpeg.decode_headers()?;
				let mut tile = Image::zero(256.into());
				if let Err(e) = jpeg.decode_into(bytemuck::cast_slice_mut::<rgb8,_>(&mut tile.data)) { println!("{e}"); }
				assert_eq!(tile.size.x*tile.size.y, tile.data.len() as _);
				let mut min_x = 0; if x0==tiles.min.x { min_x = min.x-x0*tile_size.x };
				let mut min_y = 0; if y0==tiles.min.y { min_y = min.y-y0*tile_size.y };
				let mut max_x = tile.size.x; if x0==tiles.max.x { max_x = max.x-x0*tile_size.x };
				let mut max_y = tile.size.y; if y0==tiles.max.y { max_y = max.y-y0*tile_size.y };
				for y in min_y..max_y { for x in min_x..max_x {
					let image_size_y = image.size.y;
					image[xy{x: x0*tile_size.x+x-min.x, y: image_size_y-1-(y0*tile_size.y+y-min.y)}] = tile[xy{x,y}]; //{let rgba{r,g,b,a:_} = tile[xy{x,y}]; rgb8{r,g,b}};
				}}
			}}
			std::fs::write(cache, bytemuck::cast_slice(&image.data))?;
			//image::save_rgb(format!("{}/.cache/lidar/{name}.png", std::env::var("HOME")?), &image)?;
			println!("{cache}");
		}
		let color = map::<rgb8>(cache)?;
		let color = Image::new(10000.into(), color);
		let color = image(context, commands, color.as_ref().map(|&v| rgba8::from(v)).as_ref())?;
		
		let vertex_grid_size_x = {assert_eq!(ground.size.x, ground.size.y); ground.size.x};
		let size_in_meters = vertex_grid_size_x as f32 * meters_per_pixel;
		let meters_to_normalized = 2./size_in_meters; // shader scales xy integer coordinates [0..`size`] to screen width (=2). i.e: = i/size*2-1
		fn minmax(values: &[f32]) -> [f32; 2] {
			let [mut min, mut max] = [f32::INFINITY, -f32::INFINITY];
			for &value in values { if value > f32::MIN && value < min { min = value; } if value > max { max = value; } }
			[min, max]
		}
		let [min_height, _max] = minmax(&ground.data);
		let z = |height| meters_to_normalized*(height-min_height);
		
		Ok(Self{
			terrain: Terrain::new(context, commands, &ground, meters_per_pixel, z, color.clone())?,
			points: Points::new(context, commands, points, z, color)?,
			view_position: xy{x: 0., y: 0.}, yaw: 0.
		})
	}}

impl ui::Widget for App {
	fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
		let Self{terrain, points, view_position, yaw} = self;
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
		points.render(context, commands, target.clone(), depth.clone(), view_projection)?;

		*yaw += std::f32::consts::PI/6./60.;
		Ok(())
	}
	fn event(&mut self, _context: &Context, _commands: &mut Commands, _size: size, _event_context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}

fn main() -> Result { ui::run("lidar", Box::new(move |context, commands| Ok(Box::new(App::new(context, commands)?)))) }

