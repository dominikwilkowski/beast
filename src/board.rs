use rand::seq::SliceRandom;
use std::{
	fmt::Write,
	ops::{Index, IndexMut},
};

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, LEVEL_ONE, LEVEL_THREE, LEVEL_TWO, Level, Tile};

#[derive(Debug)]
pub struct Board {
	pub data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
	pub beast_locations: Vec<Coord>,
	pub super_beast_locations: Vec<Coord>,
	pub egg_locations: Vec<Coord>,
	pub player_position: Coord,
}

impl Index<Coord> for Board {
	type Output = Tile;

	fn index(&self, coord: Coord) -> &Self::Output {
		&self.data[coord.row][coord.column]
	}
}

impl IndexMut<Coord> for Board {
	fn index_mut(&mut self, coord: Coord) -> &mut Self::Output {
		&mut self.data[coord.row][coord.column]
	}
}

impl Board {
	pub fn new(level: Level) -> Self {
		let mut data = [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

		let level_config = match level {
			Level::One => LEVEL_ONE,
			Level::Two => LEVEL_TWO,
			Level::Three => LEVEL_THREE,
		};

		let player_position = Coord {
			row: BOARD_HEIGHT - 1,
			column: 0,
		};

		let mut beast_locations = Vec::with_capacity(level_config.beasts);
		let mut super_beast_locations = Vec::with_capacity(level_config.super_beasts);
		let mut egg_locations = Vec::with_capacity(level_config.eggs);

		data[player_position.row][player_position.column] = Tile::Player;

		let mut all_positions: Vec<Coord> = (0..BOARD_HEIGHT)
			.flat_map(|y| (0..BOARD_WIDTH).map(move |x| Coord { row: y, column: x }))
			.filter(|coord| !(coord.row == BOARD_HEIGHT - 1 && coord.column == 0)) // filter out player position
			.collect();

		let total_entities =
			level_config.blocks + level_config.static_blocks + level_config.super_beasts + level_config.eggs;
		let mut rng = rand::rng();
		all_positions.shuffle(&mut rng);
		let block_positions: Vec<Coord> = all_positions.drain(0..total_entities).collect();

		for &coord in block_positions.iter().take(level_config.blocks) {
			data[coord.row][coord.column] = Tile::Block;
		}

		for &coord in block_positions.iter().skip(level_config.blocks).take(level_config.static_blocks) {
			data[coord.row][coord.column] = Tile::StaticBlock;
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
				super_beast_locations.push(coord);
				data[coord.row][coord.column] = Tile::SuperBeast;
				placed_super_beasts += 1;
			} else if placed_eggs < level_config.eggs {
				egg_locations.push(coord);
				data[coord.row][coord.column] = Tile::Egg;
				placed_eggs += 1;
			} else if placed_beasts < level_config.beasts {
				beast_locations.push(coord);
				data[coord.row][coord.column] = Tile::CommonBeast;
				placed_beasts += 1;
			}

			// skipping a couple tiles to give beasts some room
			i += level_config.beast_starting_distance;
		}

		Self {
			data,
			beast_locations,
			super_beast_locations,
			egg_locations,
			player_position,
		}
	}

	pub fn render(&self) -> String {
		let mut output = String::with_capacity(BOARD_WIDTH * BOARD_HEIGHT * 2 + BOARD_HEIGHT);

		for row in self.data.iter() {
			write!(output, "\x1b[33m▌\x1b[39m").unwrap_or_else(|_| panic!("Can't write to string buffer"));
			for tile in row.iter() {
				write!(output, "{}", tile).unwrap_or_else(|_| panic!("Can't write to string buffer"));
			}
			writeln!(output, "\x1b[33m▐\x1b[39m").unwrap_or_else(|_| panic!("Can't write to string buffer"));
		}

		output
	}
}
