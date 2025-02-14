use rand::seq::{IteratorRandom, SliceRandom};
use std::fmt::Write;

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

		let (blocks, immovable_blocks, beasts) = match level {
			Level::One => (300, 30, 3),
			Level::Two => (200, 50, 5),
			Level::Three => (50, 150, 12),
		};

		let mut all_positions: Vec<(usize, usize)> =
			(0..BOARD_HEIGHT).flat_map(|y| (0..BOARD_WIDTH).map(move |x| (y, x))).collect();

		let total_blocks_needed = blocks + immovable_blocks;
		let mut rng = rand::rng();
		all_positions.shuffle(&mut rng);
		let block_positions: Vec<(usize, usize)> = all_positions.drain(0..total_blocks_needed).collect();

		for &(row, column) in block_positions.iter().take(blocks) {
			data[row][column] = Tile::Block;
		}

		for &(row, column) in block_positions.iter().skip(blocks).take(immovable_blocks) {
			data[row][column] = Tile::ImmovableBlock;
		}

		let top_right = (0, BOARD_WIDTH - 1);
		all_positions.sort_by(|&(row1, column1), &(row2, column2)| {
			let distance_row1 = row1 as isize - top_right.0 as isize;
			let distance_column1 = column1 as isize - top_right.1 as isize;
			let distance_row2 = row2 as isize - top_right.0 as isize;
			let distance_column2 = column2 as isize - top_right.1 as isize;
			// calculating the Euclidean distance
			// distance^2 = distance_x^2+distance_y^2
			let distance1 = distance_row1 * distance_row1 + distance_column1 * distance_column1;
			let distance2 = distance_row2 * distance_row2 + distance_column2 * distance_column2;
			distance1.cmp(&distance2)
		});

		let mut placed_beasts = 0;
		let mut i = 0;
		while placed_beasts < beasts {
			if i >= all_positions.len() {
				panic!("Could not find a free spot to place all beasts");
			}

			let (row, col) = all_positions[i];
			data[row][col] = Tile::Beast;
			placed_beasts += 1;
			// skipping a couple tiles to give beasts some room
			i += 16;
		}

		data[BOARD_HEIGHT - 1][0] = Tile::Player;

		data
	}

	pub fn render_full(&self) -> String {
		let mut output = String::with_capacity(BOARD_WIDTH * BOARD_HEIGHT * 2 + BOARD_HEIGHT);

		for row in self.data.iter() {
			for tile in row.iter() {
				write!(output, "{}", tile).unwrap();
			}
			output.push('\n');
		}

		output
	}

	pub fn render_partial(&self, partials: Vec<Partial>) -> String {
		todo!()
	}
}
