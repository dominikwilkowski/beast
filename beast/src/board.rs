//! this module contains the board logic including terrain generation and rendering the board
use beast_common::levels::Level;

use rand::{Rng, seq::SliceRandom};
use std::{
	fmt::Write,
	ops::{Index, IndexMut},
	time::{Duration, Instant},
};

use crate::{
	ANSI_LEFT_BORDER, ANSI_RESET_BG, ANSI_RIGHT_BORDER, BOARD_HEIGHT, BOARD_WIDTH, Coord, PLAYER_START, Tile,
	beasts::{Beast, CommonBeast, Egg, HatchedBeast, SuperBeast},
	player::Player,
};

#[derive(Debug, Clone, Copy)]
pub struct Board {
	pub data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
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

/// data that is returned from the terrain generation to be used by the game struct
pub struct BoardTerrainInfo {
	/// the board itself
	pub data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
	/// a collection of common beasts with their position on the board
	pub common_beasts: Vec<CommonBeast>,
	/// a collection of super beasts with their position on the board
	pub super_beasts: Vec<SuperBeast>,
	/// a collection of eggs with their position on the board
	pub eggs: Vec<Egg>,
	/// a collection of hatched beasts with their position on the board
	pub hatched_beasts: Vec<HatchedBeast>,
	/// the instance player which includes their position on the board
	pub player: Player,
}

impl Board {
	pub fn new(data: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT]) -> Self {
		Self { data }
	}

	pub fn generate_terrain(level: Level) -> BoardTerrainInfo {
		let mut data = [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

		let level_config = level.get_config();

		let mut common_beasts = Vec::with_capacity(level_config.common_beasts);
		let mut super_beasts = Vec::with_capacity(level_config.super_beasts);
		let mut eggs = Vec::with_capacity(level_config.eggs);

		data[PLAYER_START.row][PLAYER_START.column] = Tile::Player;

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
				super_beasts.push(SuperBeast::new(coord));
				data[coord.row][coord.column] = Tile::SuperBeast;
				placed_super_beasts += 1;
			} else if placed_eggs < level_config.eggs {
				let mut rng = rand::rng();
				let time = Instant::now() - Duration::from_millis(rng.random_range(0..3000));
				eggs.push(Egg::new(coord, time));
				data[coord.row][coord.column] = Tile::Egg(time);
				placed_eggs += 1;
			} else if placed_beasts < level_config.common_beasts {
				common_beasts.push(CommonBeast::new(coord));
				data[coord.row][coord.column] = Tile::CommonBeast;
				placed_beasts += 1;
			}

			// skipping a couple tiles to give beasts some room
			i += level_config.beast_starting_distance;
		}

		BoardTerrainInfo {
			data,
			common_beasts,
			super_beasts,
			eggs,
			hatched_beasts: Vec::new(),
			player: Player::new(PLAYER_START),
		}
	}

	pub fn render(&self) -> String {
		let mut output = String::with_capacity(BOARD_WIDTH * BOARD_HEIGHT * 2 + BOARD_HEIGHT);

		for row in self.data.iter() {
			write!(output, "{ANSI_LEFT_BORDER}").unwrap_or_else(|_| panic!("Can't write to string buffer"));
			for tile in row.iter() {
				write!(output, "{tile}").unwrap_or_else(|_| panic!("Can't write to string buffer"));
			}
			writeln!(output, "{ANSI_RIGHT_BORDER}").unwrap_or_else(|_| panic!("Can't write to string buffer"));
		}
		write!(output, "{ANSI_RESET_BG}").unwrap_or_else(|_| panic!("Can't write to string buffer"));

		output
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use beast_common::levels::*;

	use crate::common::strip_ansi_border;

	#[test]
	fn new_level_one() {
		let info = Board::generate_terrain(Level::One);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should start at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_ONE.common_beasts,
			"The number of common beasts should match the level 1 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_ONE.super_beasts,
			"The number of super beasts should match the level 1 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_ONE.eggs, "The number of eggs should match the level 1 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 1 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_ONE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Two);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_TWO.common_beasts,
			"The number of common beasts should match the level 2 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_TWO.super_beasts,
			"The number of super beasts should match the level 2 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_TWO.eggs, "The number of eggs should match the level 2 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 2 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_TWO.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Three);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_THREE.common_beasts,
			"The number of common beasts should match the level 3 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_THREE.super_beasts,
			"The number of super beasts should match the level 3 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_THREE.eggs, "The number of eggs should match the level 3 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 3 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_THREE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Four);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_FOUR.common_beasts,
			"The number of common beasts should match the level 4 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_FOUR.super_beasts,
			"The number of super beasts should match the level 4 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_FOUR.eggs, "The number of eggs should match the level 4 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 4 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_FOUR.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Five);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_FIVE.common_beasts,
			"The number of common beasts should match the level 5 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_FIVE.super_beasts,
			"The number of super beasts should match the level 5 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_FIVE.eggs, "The number of eggs should match the level 5 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 5 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_FIVE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Six);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_SIX.common_beasts,
			"The number of common beasts should match the level 6 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_SIX.super_beasts,
			"The number of super beasts should match the level 6 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_SIX.eggs, "The number of eggs should match the level 6 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 6 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_SIX.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Seven);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_SEVEN.common_beasts,
			"The number of common beasts should match the level 7 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_SEVEN.super_beasts,
			"The number of super beasts should match the level 7 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_SEVEN.eggs, "The number of eggs should match the level 7 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 7 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_SEVEN.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Eight);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_EIGHT.common_beasts,
			"The number of common beasts should match the level 8 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_EIGHT.super_beasts,
			"The number of super beasts should match the level 8 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_EIGHT.eggs, "The number of eggs should match the level 8 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 8 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_EIGHT.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Nine);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_NINE.common_beasts,
			"The number of common beasts should match the level 9 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_NINE.super_beasts,
			"The number of super beasts should match the level 9 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_NINE.eggs, "The number of eggs should match the level 9 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 9 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_NINE.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
		let info = Board::generate_terrain(Level::Ten);
		let board = Board { data: info.data };

		assert_eq!(
			info.player.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 1
			},
			"The player should be at the bottom left corner"
		);
		assert_eq!(
			info.common_beasts.len(),
			LEVEL_TEN.common_beasts,
			"The number of common beasts should match the level 10 definition"
		);
		assert_eq!(
			info.super_beasts.len(),
			LEVEL_TEN.super_beasts,
			"The number of super beasts should match the level 10 definition"
		);
		assert_eq!(info.eggs.len(), LEVEL_TEN.eggs, "The number of eggs should match the level 10 definition");
		assert_eq!(info.hatched_beasts.len(), 0, "The number of hatched beasts should match the level 10 definition");

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
			board.data.iter().flatten().filter(|tile| matches!(tile, Tile::Egg(_))).count(),
			LEVEL_TEN.eggs,
			"There should be the right amount of egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| matches!(tile, Tile::EggHatching(_))).count(),
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
	fn render_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let render_string = board.render();
		let lines = render_string.lines();

		assert_eq!(lines.clone().count(), BOARD_HEIGHT + 1, "Board should render the right amount of lines");

		for (i, line) in lines.enumerate() {
			if i < BOARD_HEIGHT {
				assert_eq!(
					strip_ansi_border(line).len(),
					BOARD_WIDTH * 2,
					"Board line {i} should render the right amount of line lengths"
				);
			}
		}
	}

	#[test]
	fn board_terrain_generation_test() {
		let levels = [
			Level::One,
			Level::Two,
			Level::Three,
			Level::Four,
			Level::Five,
			Level::Six,
			Level::Seven,
			Level::Eight,
			Level::Nine,
			Level::Ten,
		];

		for level in &levels {
			let terrain_info = Board::generate_terrain(*level);
			let config = level.get_config();

			assert_eq!(
				terrain_info.common_beasts.len(),
				config.common_beasts,
				"Common beast count should match level config for level {level}"
			);

			assert_eq!(
				terrain_info.super_beasts.len(),
				config.super_beasts,
				"Super beast count should match level config for level {level}"
			);

			assert_eq!(terrain_info.eggs.len(), config.eggs, "Egg count should match level config for level {level}");

			assert_eq!(
				terrain_info.hatched_beasts.len(),
				0,
				"No hatched beasts should be present initially for level {level}"
			);

			assert_eq!(
				terrain_info.player.position, PLAYER_START,
				"Player should start at PLAYER_START position for level {level}",
			);

			// Check that board has correct number of each tile type
			let board = Board::new(terrain_info.data);
			let block_count = board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count();
			let static_block_count = board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count();

			assert_eq!(block_count, config.blocks, "Block count should match level config for level {level}");

			assert_eq!(
				static_block_count, config.static_blocks,
				"Static block count should match level config for level {level}"
			);
		}
	}
}
