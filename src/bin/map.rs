use papariki::geometry::LonLat;
use papariki::globe::Globe;

fn main() {
	let mut globe = Globe::new();
	let ll = LonLat::new(0.0, 52.0);
	let tile = globe.get_tile(&ll);
	//let mesh = tile.as_mesh();

	//println!("Hello, world! {:?}", tile.layers[0].name);
}
