use rand::seq::SliceRandom;
use std::{cmp::Ordering, time::Instant};

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile, board::Board, levels::LevelConfig};

// TODO: add trait for beast

pub enum BeastAction {
	PlayerKilled,
	Movement,
}

pub struct CommonBeast {
	pub position: Coord,
}

impl CommonBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	// this is the simpelest path finding that I could come up with
	// the beasts just move in your direction without looking obstacles
	pub fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		// top row
		let left_top: Coord = Coord {
			column: self.position.column.saturating_sub(1),
			row: self.position.row.saturating_sub(1),
		};
		let middle_top: Coord = Coord {
			column: self.position.column,
			row: self.position.row.saturating_sub(1),
		};
		let right_top: Coord = Coord {
			column: std::cmp::min(self.position.column + 1, BOARD_WIDTH - 1),
			row: self.position.row.saturating_sub(1),
		};

		// middle row
		let left_middle: Coord = Coord {
			column: self.position.column.saturating_sub(1),
			row: self.position.row,
		};
		let right_middle: Coord = Coord {
			column: std::cmp::min(self.position.column + 1, BOARD_WIDTH - 1),
			row: self.position.row,
		};

		// bottom row
		let left_bottom: Coord = Coord {
			column: self.position.column.saturating_sub(1),
			row: std::cmp::min(self.position.row + 1, BOARD_HEIGHT - 1),
		};
		let middle_bottom: Coord = Coord {
			column: self.position.column,
			row: std::cmp::min(self.position.row + 1, BOARD_HEIGHT - 1),
		};
		let right_bottom: Coord = Coord {
			column: std::cmp::min(self.position.column + 1, BOARD_WIDTH - 1),
			row: std::cmp::min(self.position.row + 1, BOARD_HEIGHT - 1),
		};

		let possible_moves: [Coord; 8] =
			match (player_position.column.cmp(&self.position.column), player_position.row.cmp(&self.position.row)) {
				(Ordering::Equal, Ordering::Greater) => {
					// player is straight below
					// 6 8  7
					// 4 ├┤ 5
					// 2 ◀▶ 3
					Self::shuffle_movements([
						middle_top,
						right_top,
						left_top,
						right_middle,
						left_middle,
						right_bottom,
						left_bottom,
						middle_bottom,
					])
				},
				(Ordering::Equal, Ordering::Less) => {
					// player is straight above
					// 2 ◀▶ 3
					// 4 ├┤ 5
					// 6 8  7
					Self::shuffle_movements([
						middle_bottom,
						right_bottom,
						left_bottom,
						right_middle,
						left_middle,
						right_top,
						left_top,
						middle_top,
					])
				},
				(Ordering::Less, Ordering::Equal) => {
					// player is straight left
					// 2 4  6
					// ◀▶├┤ 8
					// 3 5  7
					Self::shuffle_movements([
						right_middle,
						right_bottom,
						right_top,
						middle_bottom,
						middle_top,
						left_bottom,
						left_top,
						left_middle,
					])
				},
				(Ordering::Greater, Ordering::Equal) => {
					// player is straight right
					// 6 4  2
					// 8 ├┤◀▶
					// 7 5  3
					Self::shuffle_movements([
						left_middle,
						left_bottom,
						left_top,
						middle_bottom,
						middle_top,
						right_bottom,
						right_top,
						right_middle,
					])
				},
				(Ordering::Greater, Ordering::Greater) => {
					// player is below right
					// 8 7  5
					// 6 ├┤ 3
					// 4 2 ◀▶
					Self::shuffle_movements([
						left_top,
						middle_top,
						left_middle,
						right_top,
						left_bottom,
						right_middle,
						middle_bottom,
						right_bottom,
					])
				},
				(Ordering::Greater, Ordering::Less) => {
					// player is above right
					// 4 2 ◀▶
					// 6 ├┤ 3
					// 8 7  5
					Self::shuffle_movements([
						left_bottom,
						middle_bottom,
						left_middle,
						right_bottom,
						left_top,
						right_middle,
						middle_top,
						right_top,
					])
				},
				(Ordering::Less, Ordering::Greater) => {
					// player is below left
					// 4 6  8
					// 2 ├┤ 7
					// ◀▶ 3 5
					Self::shuffle_movements([
						right_top,
						middle_top,
						right_middle,
						right_bottom,
						left_top,
						middle_bottom,
						left_middle,
						left_bottom,
					])
				},
				(Ordering::Less, Ordering::Less) => {
					// player is above left
					// ◀▶ 3 5
					// 2 ├┤ 7
					// 4 6  8
					Self::shuffle_movements([
						right_bottom,
						right_middle,
						middle_bottom,
						right_top,
						left_bottom,
						middle_top,
						left_middle,
						left_top,
					])
				},
				(Ordering::Equal, Ordering::Equal) => {
					// Player is at the same position.
					[
						left_top,
						middle_top,
						right_top,
						left_middle,
						right_middle,
						left_bottom,
						middle_bottom,
						right_bottom,
					]
				},
			};

		for coord in possible_moves {
			match board[coord] {
				Tile::Player => {
					board[coord] = Tile::CommonBeast;
					board[self.position] = Tile::Empty;
					self.position = coord;
					return BeastAction::PlayerKilled;
				},
				Tile::Empty => {
					board[coord] = Tile::CommonBeast;
					board[self.position] = Tile::Empty;
					self.position = coord;
				},
				Tile::Block
				| Tile::StaticBlock
				| Tile::CommonBeast
				| Tile::SuperBeast
				| Tile::HatchedBeast
				| Tile::Egg(_)
				| Tile::EggHatching(_) => {
					// We can't move here
				},
			}
		}
		BeastAction::Movement
	}

	fn shuffle_movements(coords: [Coord; 8]) -> [Coord; 8] {
		let mut rng = rand::rng();

		let mut shuffled = [coords[0]; 8];

		let mut pair = [coords[1], coords[2]];
		pair.shuffle(&mut rng);
		shuffled[1] = pair[0];
		shuffled[2] = pair[1];

		let mut pair = [coords[3], coords[4]];
		pair.shuffle(&mut rng);
		shuffled[3] = pair[0];
		shuffled[4] = pair[1];

		let mut pair = [coords[5], coords[6]];
		pair.shuffle(&mut rng);
		shuffled[5] = pair[0];
		shuffled[6] = pair[1];

		shuffled[7] = coords[7];

		shuffled
	}

	pub fn get_score() -> u16 {
		2
	}
}

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn advance(&mut self, _board: &mut Board, _player_position: Coord) -> BeastAction {
		BeastAction::Movement
	}

	pub fn get_score() -> u16 {
		6
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HatchingState {
	Incubating,
	Hatching(Coord, Instant),
	Hatched(Coord),
}

pub struct Egg {
	pub position: Coord,
	pub instant: Instant,
}

impl Egg {
	pub fn new(position: Coord, instant: Instant) -> Self {
		Self { position, instant }
	}

	pub fn hatch(&self, level: LevelConfig) -> HatchingState {
		if self.instant.elapsed() >= level.egg_hatching_time {
			HatchingState::Hatched(self.position)
		} else if self.instant.elapsed() >= (level.egg_hatching_time / 10) * 8 {
			HatchingState::Hatching(self.position, self.instant)
		} else {
			HatchingState::Incubating
		}
	}

	pub fn get_score() -> u16 {
		2
	}
}

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn advance(&mut self, _board: &mut Board, _player_position: Coord) -> BeastAction {
		BeastAction::Movement
	}

	pub fn get_score() -> u16 {
		2
	}
}
