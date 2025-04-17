#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
dependencies={ gdal='*',  bytemuck='*' }
---
fn main() -> Result<(), Box<dyn std::error::Error>> {
	use gdal::{Dataset, vector::LayerAccess};
	let dataset = Dataset::open(format!("{}/swissbuildings3d_3_0_2019_1091-23_2056_5728.gdb.zip", std::env::var("HOME")?))?;
	let mut quads = vec![];
	let mut layer = dataset.layer(1)?;
	for feature in layer.features() {
		let mesh = feature.geometry().unwrap();
		for i in 0..mesh.geometry_count() {
			let face = mesh.get_geometry(i);
			if face.geometry_type() == gdal::vector::OGRwkbGeometryType::wkbTriangleZ {
				assert_eq!(face.geometry_count(), 1);
				face.get_geometry(0).get_points(&mut quads);
				assert_eq!(quads.len()%4, 0);
				/*let mut quad = vec![];
				face.get_geometry(0).get_points(&mut quad);
				assert_eq!(quad.len(), 4);
				if quad.len() == 4 { quads.append(&mut quad); }
				else { println!("{}", quad.len()); }*/
			}
		}
	}
	println!("{:?}", quads.len());
	let ref cache = format!("{}/.cache/lidar/buildings", std::env::var("HOME")?);
	let points = quads.into_iter().map(|(x,y,z)| [x as f32, y as f32, z as f32]).collect::<Box<[_]>>();
	std::fs::write(cache, bytemuck::cast_slice(&points))?;
	Ok(())
}
