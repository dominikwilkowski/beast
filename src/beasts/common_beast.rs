use rand::seq::SliceRandom;

use crate::{
	Coord, Tile,
	beasts::{Beast, BeastAction, get_walkable_coords},
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
		let possible_moves = Self::shuffle_movements(get_walkable_coords(board, &self.position, &player_position, false));

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
