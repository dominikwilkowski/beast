use rand::seq::SliceRandom;

use crate::{
	Coord, Tile,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct CommonBeast {
	pub position: Coord,
}

impl CommonBeast {
	fn shuffle_movements(mut coords: Vec<Coord>) -> Vec<Coord> {
		let mut rng = rand::rng();
		coords[1..3].shuffle(&mut rng);
		coords[3..5].shuffle(&mut rng);
		coords[5..7].shuffle(&mut rng);
		coords
	}
}

impl Beast for CommonBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	// this is the simplest path finding that I could come up with
	// the beasts just move in your direction without looking obstacles
	// this means they can get stuck behind a flat wall
	// which can be fun to play with in early levels
	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		let possible_moves =
			Self::shuffle_movements(Self::get_walkable_coords(board, &self.position, &player_position, false));

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
					return BeastAction::Moved;
				},
				Tile::Block
				| Tile::StaticBlock
				| Tile::CommonBeast
				| Tile::SuperBeast
				| Tile::HatchedBeast
				| Tile::Egg(_)
				| Tile::EggHatching(_) => {
					// we can't move here
				},
			}
		}
		BeastAction::Moved
	}

	fn get_score() -> u16 {
		2
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{BOARD_HEIGHT, BOARD_WIDTH};

	#[test]
	fn common_beast_new_test() {
		let position = Coord { column: 3, row: 4 };
		let beast = CommonBeast::new(position);
		assert_eq!(beast.position, position, "We have created a new instance of CommonBeast");
	}

	#[test]
	fn shuffle_movements_test() {
		let coords: Vec<Coord> = (0..8).map(|i| Coord { column: i, row: 5 }).collect();
		let shuffled = CommonBeast::shuffle_movements(coords.clone());

		assert_eq!(shuffled[0], coords[0], "The first element should remain the same");

		let mut original_slice = coords[1..3].to_vec();
		let mut shuffled_slice = shuffled[1..3].to_vec();
		original_slice.sort();
		shuffled_slice.sort();
		assert_eq!(original_slice, shuffled_slice, "The next two should be element 1 and 2 in any order");

		let mut original_slice = coords[3..5].to_vec();
		let mut shuffled_slice = shuffled[3..5].to_vec();
		original_slice.sort();
		shuffled_slice.sort();
		assert_eq!(original_slice, shuffled_slice, "The next two should be element 3 and 4 in any order");

		let mut original_slice = coords[5..7].to_vec();
		let mut shuffled_slice = shuffled[5..7].to_vec();
		original_slice.sort();
		shuffled_slice.sort();
		assert_eq!(original_slice, shuffled_slice, "The next two should be element 5 and 6 in any order");

		assert_eq!(shuffled[7], coords[7], "The last element should remain the same");
	}

	#[test]
	fn advance_above() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 5, row: 3 };
		board[Coord { column: 5, row: 3 }] = Tile::Player;
		let mut beast = CommonBeast::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::CommonBeast;

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Player, "The player hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::CommonBeast, "The beast has moved up");
		assert_eq!(beast.position, Coord { column: 5, row: 4 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::PlayerKilled, "The beast has killed");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::CommonBeast, "The beast has moved up");
		assert_eq!(beast.position, Coord { column: 5, row: 3 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Empty, "The previous beast tile has been cleared");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
	}

	#[test]
	fn advance_right() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 7, row: 5 };
		board[Coord { column: 7, row: 5 }] = Tile::Player;
		let mut beast = CommonBeast::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::CommonBeast;

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(board[Coord { column: 7, row: 5 }], Tile::Player, "The player hasn't moved");
		assert_eq!(board[Coord { column: 6, row: 5 }], Tile::CommonBeast, "The beast has moved right");
		assert_eq!(beast.position, Coord { column: 6, row: 5 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::PlayerKilled, "The beast has killed");
		assert_eq!(board[Coord { column: 7, row: 5 }], Tile::CommonBeast, "The beast has moved up");
		assert_eq!(beast.position, Coord { column: 7, row: 5 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 6, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
	}

	#[test]
	fn advance_below() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 5, row: 7 };
		board[Coord { column: 5, row: 7 }] = Tile::Player;
		let mut beast = CommonBeast::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::CommonBeast;

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(board[Coord { column: 5, row: 7 }], Tile::Player, "The player hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 6 }], Tile::CommonBeast, "The beast has moved down");
		assert_eq!(beast.position, Coord { column: 5, row: 6 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::PlayerKilled, "The beast has killed");
		assert_eq!(board[Coord { column: 5, row: 7 }], Tile::CommonBeast, "The beast has moved up");
		assert_eq!(beast.position, Coord { column: 5, row: 7 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 6 }], Tile::Empty, "The previous beast tile has been cleared");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
	}

	#[test]
	fn advance_left() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 3, row: 5 };
		board[Coord { column: 3, row: 5 }] = Tile::Player;
		let mut beast = CommonBeast::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::CommonBeast;

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(board[Coord { column: 3, row: 5 }], Tile::Player, "The player hasn't moved");
		assert_eq!(board[Coord { column: 4, row: 5 }], Tile::CommonBeast, "The beast has moved left");
		assert_eq!(beast.position, Coord { column: 4, row: 5 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::PlayerKilled, "The beast has killed");
		assert_eq!(board[Coord { column: 3, row: 5 }], Tile::CommonBeast, "The beast has moved up");
		assert_eq!(beast.position, Coord { column: 3, row: 5 }, "The beast coord has been recorded");
		assert_eq!(board[Coord { column: 4, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "The previous beast tile has been cleared");
	}

	#[test]
	fn advance_nowhere() {
		let mut board = Board::new([[Tile::Block; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 3, row: 5 };
		board[Coord { column: 3, row: 5 }] = Tile::Player;
		let mut beast = CommonBeast::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::CommonBeast;

		let action = beast.advance(&mut board, player_position);

		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(board[Coord { column: 3, row: 5 }], Tile::Player, "The player hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::CommonBeast, "The beast hasn't moved");
		assert_eq!(beast.position, Coord { column: 5, row: 5 }, "The beast coord are unchanged");
	}

	#[test]
	fn get_score_test() {
		assert_eq!(CommonBeast::get_score(), 2, "CommonBeast score should be 6");
	}
}
