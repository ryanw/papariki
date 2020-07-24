use papariki::geometry::LonLat;
use papariki::globe::Globe;


fn main() {

	let mut globe = Globe::new();
	let tile = globe.get_tile(&LonLat::new(0.0, 52.0));
	let mesh = tile.as_mesh();

	//println!("Hello, world! {:?}", tile.layers[0].name);

}
