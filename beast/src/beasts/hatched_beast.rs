use std::cmp::Ordering;

use crate::{
	BOARD_HEIGHT,
	BOARD_WIDTH,
	Coord,
	// Dir,
	Tile,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	fn try_squish_player(&mut self, board: &mut Board, player_position: Coord) -> Option<(Coord, Coord)> {
		// the manhattan distance (https://en.wikipedia.org/wiki/Taxicab_geometry)
		let distance_column = self.position.column.abs_diff(player_position.column);
		let distance_row = self.position.row.abs_diff(player_position.row);
		if distance_column + distance_row <= 2 {
			match (player_position.column.cmp(&self.position.column), player_position.row.cmp(&self.position.row)) {
				(Ordering::Equal, Ordering::Greater) => {
					// player is straight below
					// ├┤
					// ◀▶
					let beast_coord = Coord {
						column: self.position.column,
						row: std::cmp::min(self.position.row + 1, BOARD_HEIGHT - 1),
					};
					let block_coord = Coord {
						column: self.position.column,
						row: std::cmp::min(self.position.row + 2, BOARD_HEIGHT - 1),
					};
					if board[beast_coord] == Tile::Block {
						return Some((beast_coord, block_coord));
					}
				},
				(Ordering::Equal, Ordering::Less) => {
					// player is straight above
					// ◀▶
					// ├┤
					let beast_coord = Coord {
						column: self.position.column,
						row: self.position.row.saturating_sub(1),
					};
					let block_coord = Coord {
						column: self.position.column,
						row: self.position.row.saturating_sub(2),
					};
					if board[beast_coord] == Tile::Block {
						return Some((beast_coord, block_coord));
					}
				},
				(Ordering::Less, Ordering::Equal) => {
					// player is straight left
					// ◀▶├┤
					let beast_coord = Coord {
						column: self.position.column.saturating_sub(1),
						row: self.position.row,
					};
					let block_coord = Coord {
						column: self.position.column.saturating_sub(2),
						row: self.position.row,
					};
					if board[beast_coord] == Tile::Block {
						return Some((beast_coord, block_coord));
					}
				},
				(Ordering::Greater, Ordering::Equal) => {
					// player is straight right
					// ├┤◀▶
					let beast_coord = Coord {
						column: std::cmp::min(self.position.column + 1, BOARD_WIDTH - 1),
						row: self.position.row,
					};
					let block_coord = Coord {
						column: std::cmp::min(self.position.column + 2, BOARD_WIDTH - 1),
						row: self.position.row,
					};
					if board[beast_coord] == Tile::Block {
						return Some((beast_coord, block_coord));
					}
				},
				(Ordering::Greater, Ordering::Greater)
				| (Ordering::Greater, Ordering::Less)
				| (Ordering::Less, Ordering::Greater)
				| (Ordering::Less, Ordering::Less)
				| (Ordering::Equal, Ordering::Equal) => {
					// player is at a position that is too far away to squish
				},
			}
		}

		None
	}
}

impl Beast for HatchedBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		// 1. check if you can squish the player immediately as the next move
		if let Some((beast_coord, block_coord)) = self.try_squish_player(board, player_position) {
			board[beast_coord] = Tile::HatchedBeast;
			board[self.position] = Tile::Empty;
			self.position = beast_coord;
			board[block_coord] = Tile::Block;
			return BeastAction::PlayerKilled;
		}

		// 2. path find to player using a*
		if let Some(path) = Self::astar(board, self.position, &player_position) {
			if path.len() > 1 {
				// the first item is our own position
				let next_step = path[1];

				match board[next_step] {
					Tile::Player => {
						board[next_step] = Tile::HatchedBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::PlayerKilled;
					},
					Tile::Empty => {
						board[next_step] = Tile::HatchedBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::Moved;
					},
					_ => {},
				}
			}
		}

		// 3. if no a* path: try to find a way to push a block
		// - plot straight path
		// - loop
		//     - move a block you hit
		//     - check with a* if we now get a path
		// - make path thicker
		// - loop again
		// - when block found that needs moving check if beast is there or if we have to go there now

		// 4. if nothing works walk towards player via get_walkable_coords()

		BeastAction::Moved
	}

	fn get_score() -> u16 {
		2
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn try_squish_player_straight_below() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 5, row: 7 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 8 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		assert_eq!(
			beast.try_squish_player(&mut board, player_position),
			Some((Coord { column: 5, row: 6 }, player_position))
		);
	}

	#[test]
	fn try_squish_player_straight_above() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 5, row: 3 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		assert_eq!(
			beast.try_squish_player(&mut board, player_position),
			Some((Coord { column: 5, row: 4 }, player_position))
		);
	}

	#[test]
	fn try_squish_player_straight_left() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 3, row: 5 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 2, row: 5 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		assert_eq!(
			beast.try_squish_player(&mut board, player_position),
			Some((Coord { column: 4, row: 5 }, player_position))
		);
	}

	#[test]
	fn try_squish_player_straight_right() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 7, row: 5 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 8, row: 5 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		assert_eq!(
			beast.try_squish_player(&mut board, player_position),
			Some((Coord { column: 6, row: 5 }, player_position))
		);
	}
}
