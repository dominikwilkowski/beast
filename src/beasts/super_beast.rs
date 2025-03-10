use crate::{
	Coord,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct SuperBeast {
	pub position: Coord,
}

impl Beast for SuperBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, _board: &mut Board, _player_position: Coord) -> BeastAction {
		BeastAction::Moved
	}

	fn get_score() -> u16 {
		6
	}
}
