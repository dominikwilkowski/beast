use crate::{Coord, Tile, BOARD_HEIGHT, BOARD_WIDTH};

pub struct Partial<'a> {
	coord: Coord,
	replacement: &'a str,
}

pub struct Board {
	data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
	pub fn new() -> Self {
		Self {}
	}

	fn terrain_gen() {}

	pub fn render_full() {}
	pub fn render_partial(partials: Vec<Partial>) {}
}
