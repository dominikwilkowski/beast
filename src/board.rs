use crate::{Coord, Level, Tile, BOARD_HEIGHT, BOARD_WIDTH};

pub struct Partial<'a> {
	coord: Coord,
	replacement: &'a str,
}

pub struct Board {
	data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
	pub fn new() -> Self {
		Self {
			data: Self::terrain_gen(Level::One),
		}
	}

	fn terrain_gen(level: Level) -> [[Tile; BOARD_WIDTH]; BOARD_HEIGHT] {
		let mut data = [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

		data
	}

	pub fn render_full() {}
	pub fn render_partial(partials: Vec<Partial>) {}
}
