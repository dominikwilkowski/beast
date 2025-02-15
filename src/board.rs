use rand::seq::SliceRandom;
use std::fmt::Write;

use crate::{Coord, Level, Tile, BOARD_HEIGHT, BOARD_WIDTH, LEVEL_ONE, LEVEL_THREE, LEVEL_TWO};

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

		let level_config = match level {
			Level::One => LEVEL_ONE,
			Level::Two => LEVEL_TWO,
			Level::Three => LEVEL_THREE,
		};

		data[BOARD_HEIGHT - 1][0] = Tile::Player;

		let mut all_positions: Vec<Coord> = (0..BOARD_HEIGHT)
			.flat_map(|y| (0..BOARD_WIDTH).map(move |x| Coord { row: y, column: x }))
			.filter(|coord| !(coord.row == BOARD_HEIGHT - 1 && coord.column == 0)) // filter out player position
			.collect();

		let total_blocks_needed =
			level_config.blocks + level_config.immovable_blocks + level_config.super_beasts + level_config.eggs;
		let mut rng = rand::rng();
		all_positions.shuffle(&mut rng);
		let block_positions: Vec<Coord> = all_positions.drain(0..total_blocks_needed).collect();

		for &coord in block_positions.iter().take(level_config.blocks) {
			data[coord.row][coord.column] = Tile::Block;
		}

		for &coord in block_positions.iter().skip(level_config.blocks).take(level_config.immovable_blocks) {
			data[coord.row][coord.column] = Tile::ImmovableBlock;
		}

		let top_right = Coord {
			row: 0,
			column: BOARD_WIDTH - 1,
		};
		all_positions.sort_by(|coord1, coord2| {
			let distance_row1 = coord1.row as isize - top_right.row as isize;
			let distance_column1 = coord1.column as isize - top_right.column as isize;
			let distance_row2 = coord2.row as isize - top_right.row as isize;
			let distance_column2 = coord2.column as isize - top_right.column as isize;
			// calculating the Euclidean distance
			// distance^2 = distance_x^2+distance_y^2
			let distance1 = distance_row1 * distance_row1 + distance_column1 * distance_column1;
			let distance2 = distance_row2 * distance_row2 + distance_column2 * distance_column2;
			distance1.cmp(&distance2)
		});

		let mut placed_beasts = 0;
		let mut placed_super_beasts = 0;
		let mut placed_eggs = 0;
		let mut i = 0;
		while placed_beasts + placed_super_beasts + placed_eggs
			< level_config.beasts + level_config.super_beasts + level_config.eggs
		{
			if i >= all_positions.len() {
				panic!("Could not find a free spot to place all beasts");
			}

			let coord = all_positions[i];
			if placed_super_beasts < level_config.super_beasts {
				data[coord.row][coord.column] = Tile::SuperBeast;
				placed_super_beasts += 1;
			} else if placed_eggs < level_config.eggs {
				data[coord.row][coord.column] = Tile::Egg;
				placed_eggs += 1;
			} else if placed_beasts < level_config.beasts {
				data[coord.row][coord.column] = Tile::Beast;
				placed_beasts += 1;
			}

			// skipping a couple tiles to give beasts some room
			i += level_config.beast_starting_distance;
		}

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
