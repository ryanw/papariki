use futures::executor::block_on;

use papariki::globe::Globe;

fn main() {
	let globe = Globe::new("");
	let tile = block_on(globe.get_tile(0, 0, 1));
	let _verts: Vec<f32> = tile.vertices();
}
