use rand::seq::SliceRandom;
use std::{
	fmt::Write,
	ops::{Index, IndexMut},
};

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile, levels::Level};

#[derive(Debug)]
pub struct Board {
	pub data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
	pub common_beast_locations: Vec<Coord>,
	pub super_beast_locations: Vec<Coord>,
	pub egg_locations: Vec<Coord>,
	pub hatching_egg_locations: Vec<Coord>,
	pub hatched_beast_locations: Vec<Coord>,
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

		let level_config = level.get_config();

		let player_position = Coord {
			column: 0,
			row: BOARD_HEIGHT - 1,
		};

		let mut common_beast_locations = Vec::with_capacity(level_config.common_beasts);
		let mut super_beast_locations = Vec::with_capacity(level_config.super_beasts);
		let mut egg_locations = Vec::with_capacity(level_config.eggs);

		data[player_position.row][player_position.column] = Tile::Player;

		let mut all_positions: Vec<Coord> = (0..BOARD_HEIGHT)
			.flat_map(|y| (0..BOARD_WIDTH).map(move |x| Coord { column: x, row: y }))
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
			column: BOARD_WIDTH - 1,
			row: 0,
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
			< level_config.common_beasts + level_config.super_beasts + level_config.eggs
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
			} else if placed_beasts < level_config.common_beasts {
				common_beast_locations.push(coord);
				data[coord.row][coord.column] = Tile::CommonBeast;
				placed_beasts += 1;
			}

			// skipping a couple tiles to give beasts some room
			i += level_config.beast_starting_distance;
		}

		Self {
			data,
			common_beast_locations,
			super_beast_locations,
			egg_locations,
			hatching_egg_locations: Vec::new(),
			hatched_beast_locations: Vec::new(),
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::levels::*;

	#[test]
	fn new_level_one() {
		let board = Board::new(Level::One);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_ONE.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_ONE.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_ONE.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_ONE.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_ONE.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_ONE.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_ONE.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_ONE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_two() {
		let board = Board::new(Level::Two);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_TWO.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_TWO.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_TWO.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_TWO.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_TWO.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_TWO.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_TWO.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_TWO.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_three() {
		let board = Board::new(Level::Three);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_THREE.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_THREE.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_THREE.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_THREE.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_THREE.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_THREE.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_THREE.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_THREE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_four() {
		let board = Board::new(Level::Four);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_FOUR.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_FOUR.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_FOUR.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_FOUR.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_FOUR.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_FOUR.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_FOUR.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_FOUR.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_five() {
		let board = Board::new(Level::Five);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_FIVE.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_FIVE.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_FIVE.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_FIVE.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_FIVE.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_FIVE.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_FIVE.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_FIVE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_six() {
		let board = Board::new(Level::Six);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_SIX.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_SIX.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_SIX.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_SIX.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_SIX.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_SIX.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_SIX.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_SIX.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_seven() {
		let board = Board::new(Level::Seven);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_SEVEN.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_SEVEN.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_SEVEN.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_SEVEN.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_SEVEN.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_SEVEN.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_SEVEN.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_SEVEN.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_eight() {
		let board = Board::new(Level::Eight);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_EIGHT.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_EIGHT.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_EIGHT.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_EIGHT.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_EIGHT.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_EIGHT.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_EIGHT.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_EIGHT.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_nine() {
		let board = Board::new(Level::Nine);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_NINE.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_NINE.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_NINE.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_NINE.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_NINE.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_NINE.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_NINE.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_NINE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}

	#[test]
	fn new_level_ten() {
		let board = Board::new(Level::Ten);

		assert_eq!(
			board.player_position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			}
		);
		assert_eq!(board.common_beast_locations.len(), LEVEL_TEN.common_beasts);
		assert_eq!(board.super_beast_locations.len(), LEVEL_TEN.super_beasts);
		assert_eq!(board.egg_locations.len(), LEVEL_TEN.eggs);
		assert_eq!(board.hatching_egg_locations.len(), 0);
		assert_eq!(board.hatched_beast_locations.len(), 0);

		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			LEVEL_TEN.blocks,
			"There should be the right amount of block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			LEVEL_TEN.static_blocks,
			"There should be the right amount of static block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			LEVEL_TEN.common_beasts,
			"There should be the right amount of common beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			LEVEL_TEN.super_beasts,
			"There should be the right amount of super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Egg).count(),
			LEVEL_TEN.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::EggHatching).count(),
			0,
			"There should be the right amount of egg hatching tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be the right amount of hatched beast tiles"
		);
	}
}
