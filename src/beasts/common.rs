use crate::{Coord, board::Board};

/// the action a beast can take
pub enum BeastAction {
	/// the beast has killed the player
	PlayerKilled,
	/// the beast has moved to a new position
	Moved,
}

/// this trait defines the common behavior of all beasts in the game
pub trait Beast {
	/// creates a new instance of the beast and stores its position
	fn new(position: Coord) -> Self;

	/// advances the beast's position and returns the action taken
	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction;

	/// returns the score for when this beast is crushed
	fn get_score() -> u16;
}
