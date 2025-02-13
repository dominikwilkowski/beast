use rand::seq::IteratorRandom;

use crate::{Coord, Level, Tile, BOARD_HEIGHT, BOARD_WIDTH};

pub struct Partial<'a> {
	coord: Coord,
	replacement: &'a str,
}

#[derive(Debug)]
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

		let mut rng = rand::rng();

		let (blocks, immovable_blocks, _beasts) = match level {
			Level::One => (600, 60, 3),
			Level::Two => (400, 100, 5),
			Level::Three => (150, 250, 12),
		};

		let all_coords = (0..BOARD_HEIGHT).flat_map(|y| (0..BOARD_WIDTH).map(move |x| (y, x)));
		let total_needed = blocks + immovable_blocks;
		let coords: Vec<(usize, usize)> = all_coords.choose_multiple(&mut rng, total_needed);

		for &(y, x) in coords.iter().take(blocks) {
			data[y][x] = Tile::Block;
		}

		for &(y, x) in coords.iter().skip(blocks).take(immovable_blocks) {
			data[y][x] = Tile::ImmovableBlock;
		}

		data
	}

	pub fn render_full() {}
	pub fn render_partial(partials: Vec<Partial>) {}
}
