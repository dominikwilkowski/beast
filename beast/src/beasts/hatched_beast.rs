use crate::{
	Coord,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct HatchedBeast {
	pub position: Coord,
}

impl Beast for HatchedBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	// TODO: implement pathfinding that biases the beast to squash the player with blocks
	fn advance(&mut self, _board: &mut Board, _player_position: Coord) -> BeastAction {
		BeastAction::Moved
	}

	fn get_score() -> u16 {
		2
	}
}
