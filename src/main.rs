#![allow(non_snake_case)] 
//use ui::{Error, Result, throws};
pub type Error = Box<dyn core::error::Error>;
pub type Result<T=(),E=Error> = core::result::Result<T, E>;
use fehler::throws;

use owning_ref::OwningRef;
type Array<T=f32> = OwningRef<Box<memmap::Mmap>, [T]>;
#[throws] fn map<T:bytemuck::Pod>(path: &str) -> Array<T> {
    OwningRef::new(Box::new(unsafe{memmap::Mmap::map(&std::fs::File::open(path)?)}?)).map(|data| bytemuck::cast_slice(&*data))
}

mod terrain; //use terrain::Terrain;

//use ui::{Result, time};
use vector::{xyz, MinMax, vec3};//xy, size, int2, vec2, xyz, vec3, rotate, xyzw, vec4, mat4};
//use ui::vulkan::{Context, Commands, Arc, ImageView, Image, default, ImageCreateInfo, Format, ImageUsage};

struct App {
	//terrain: Terrain,
	//view_position: vec2,
	//yaw: f32,
}

impl App {
	/*fn new(context: &Context, commands: &mut Commands) -> Result<Self> {
	Ok(Self{
	terrain: Terrain::new(context, commands, ground, water, meters_per_pixel, z, color)?,
	//trees: Trees::new(context, /*ground*/water, meters_to_normalized, z)?,
	view_position: xy{x: 0., y: 0.}, yaw: 0.
	})
	}*/
}

/*impl ui::Widget for App {
fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
let Self{terrain, /*view_position, yaw*/} = self;
/*let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
let aspect_ratio = image_size.x as f32/image_size.y as f32;

let view_projection = |xyz{x,y,z}:vec3| {
let xy{x,y} = rotate(*yaw, xy{x,y} - *view_position);
let xy{x: y, y: z} = rotate(-std::f32::consts::PI/3., xy{x: y, y: z});
let z = (z-1.)/2.;
let n = 1./4.;
let f = 1.;
if true { xyzw{x, y: aspect_ratio*y, z: -z, w: 1.}  } else {
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

let depth = ImageView::new_default(Image::new(memory_allocator.clone(), ImageCreateInfo{
format: Format::D16_UNORM,
extent: target.image().extent(),
usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
..default()
}, default())?)?;

terrain.render(context, commands, target.clone(), depth.clone(), view_projection)?;*/

// *yaw += std::f32::consts::PI/6./60.;
Ok(())
}
fn event(&mut self, _size: size, _context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}*/

//fn main() -> Result { ui::run("lidar", Box::new(move |context, commands| Ok(Box::new(time("init", || App::new(context, commands))?)))) }

fn main() -> Result {
	let name = "2684500_1248500";
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
			let x = (E-center.x)/extent;
	        let y = (N-center.y)/extent;
	        let z = (z-center.z)/extent;
			//if x > -1./2. && x < 1./2. && y > -1./2. && y < 1./2. {
			points.push(xyz{x,y,z}.into());
		}
		std::fs::write(cache, bytemuck::cast_slice::<vec3,_>(&points))?;
		println!("{cache}");
	}
	dbg!(map::<vec3>(cache));
	Ok(())
}
