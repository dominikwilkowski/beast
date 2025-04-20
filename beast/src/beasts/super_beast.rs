use std::collections::HashMap;

use crate::{
	Coord, Tile,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	fn astar(board: &Board, start: Coord, goal: &Coord) -> Option<Vec<Coord>> {
		let mut open_set = vec![start];

		let mut came_from: HashMap<Coord, Coord> = HashMap::new();

		let mut g_score: HashMap<Coord, i32> = HashMap::new();
		g_score.insert(start, 0);

		let mut f_score: HashMap<Coord, i32> = HashMap::new();
		f_score.insert(start, Self::heuristic(&start, goal));

		while !open_set.is_empty() {
			let current = *open_set.iter().min_by_key(|coord| f_score.get(coord).unwrap_or(&i32::MAX)).unwrap();

			if current == *goal {
				return Some(Self::reconstruct_path(&came_from, current));
			}

			open_set.retain(|&c| c != current);

			for neighbor in Self::get_walkable_coords(board, &current, goal, true) {
				let tentative_g_score = g_score.get(&current).unwrap_or(&i32::MAX) + 1;
				if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
					came_from.insert(neighbor, current);
					g_score.insert(neighbor, tentative_g_score);
					f_score.insert(neighbor, tentative_g_score + Self::heuristic(&neighbor, goal));
					if !open_set.contains(&neighbor) {
						open_set.push(neighbor);
					}
				}
			}
		}

		None
	}
}

impl Beast for SuperBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		if let Some(path) = Self::astar(board, self.position, &player_position) {
			if path.len() > 1 {
				// the first item is our own position
				let next_step = path[1];

				match board[next_step] {
					Tile::Player => {
						board[next_step] = Tile::SuperBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::PlayerKilled;
					},
					Tile::Empty => {
						board[next_step] = Tile::SuperBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::Moved;
					},
					_ => {},
				}
			}
		} else {
			// when there is no path we at least still go towards the player
			for neighbor in Self::get_walkable_coords(board, &self.position, &player_position, true) {
				match board[neighbor] {
					Tile::Empty | Tile::Player => match board[neighbor] {
						Tile::Player => {
							board[neighbor] = Tile::SuperBeast;
							board[self.position] = Tile::Empty;
							self.position = neighbor;
							return BeastAction::PlayerKilled;
						},
						Tile::Empty => {
							board[neighbor] = Tile::SuperBeast;
							board[self.position] = Tile::Empty;
							self.position = neighbor;
							return BeastAction::Moved;
						},
						_ => {},
					},
					Tile::Block
					| Tile::StaticBlock
					| Tile::CommonBeast
					| Tile::SuperBeast
					| Tile::HatchedBeast
					| Tile::Egg(_)
					| Tile::EggHatching(_) => {},
				}
			}
		}

		BeastAction::Stayed
	}

	fn get_score() -> u16 {
		6
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{BOARD_HEIGHT, BOARD_WIDTH};

	#[test]
	fn super_beast_new_test() {
		let position = Coord { column: 3, row: 4 };
		let beast = SuperBeast::new(position);
		assert_eq!(beast.position, position, "We have created a new instance of SuperBeast");
	}

	#[test]
	fn astar_direct_path_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		let path = SuperBeast::astar(&board, beast_position, &player_position);

		assert!(path.is_some(), "Path should be calculated");
		let unwrapped_path = path.unwrap();
		assert_eq!(unwrapped_path[0], beast_position, "Path should start from the beast's position");
		assert_eq!(unwrapped_path.last().unwrap(), &player_position, "Path should end at the player's position");
	}

	#[test]
	fn astar_path_around_obstacle_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		// partial wall
		for row in 0..3 {
			board[Coord { column: 2, row }] = Tile::StaticBlock;
		}

		let path = SuperBeast::astar(&board, beast_position, &player_position);

		assert!(path.is_some(), "A* should find a path around the wall");
		let unwrapped_path = path.unwrap();
		assert_eq!(unwrapped_path[0], beast_position, "Path should start from the beast's position");
		assert_eq!(unwrapped_path.last().unwrap(), &player_position, "Path should end at the player's position");

		// Verify that the path goes around the obstacle (no coordinates with column = 2)
		for coord in &unwrapped_path {
			if coord.row < 3 {
				assert_ne!(coord.column, 2, "Path should not go through the obstacle");
			}
		}
	}

	#[test]
	fn advance_player_adjacent_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 1, row: 1 };
		let player_position = Coord { column: 3, row: 1 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		let mut beast = SuperBeast::new(beast_position);
		let result = beast.advance(&mut board, player_position);

		assert_eq!(result, BeastAction::Moved, "The beast has moved");
		assert_eq!(
			beast.position,
			Coord { column: 2, row: 1 },
			"The beast should have moved towards the player's position"
		);
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::SuperBeast, "The previous player tile is now the beast tile");
		assert_eq!(board[Coord { column: 1, row: 1 }], Tile::Empty, "The previous player tile is now cleared");

		let result = beast.advance(&mut board, player_position);

		assert_eq!(result, BeastAction::PlayerKilled, "The player was killed by the beast");
		assert_eq!(beast.position, player_position, "The beast should move to the player's position");
		assert_eq!(board[player_position], Tile::SuperBeast, "The previous player tile is now the beast tile");
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::Empty, "The previous player tile is now cleared");
	}

	#[test]
	fn advance_move_towards_player_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		let mut beast = SuperBeast::new(beast_position);
		let result = beast.advance(&mut board, player_position);

		assert_eq!(result, BeastAction::Moved, "Beast moved towards player");
		assert_ne!(beast.position, beast_position, "Beast should have moved");
		assert_eq!(board[beast.position], Tile::SuperBeast, "Beast tile was placed correctly");
		assert_eq!(board[beast_position], Tile::Empty, "Beast tile was replaced correctly");

		let old_distance = SuperBeast::heuristic(&beast_position, &player_position);
		let new_distance = SuperBeast::heuristic(&beast.position, &player_position);
		assert!(new_distance < old_distance, "New position should be closer to player");
	}

	#[test]
	fn advance_completely_surrounded_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 1, row: 1 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		board[Coord { column: 0, row: 0 }] = Tile::Block;
		board[Coord { column: 0, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 0 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let mut beast = SuperBeast::new(beast_position);
		let original_position = beast.position;
		let result = beast.advance(&mut board, player_position);

		assert_eq!(result, BeastAction::Stayed, "Beast should return Stayed");
		assert_eq!(beast.position, original_position, "Position shouldn't have changed as it's completely surrounded");
		assert_eq!(board[beast_position], Tile::SuperBeast, "Board state should remain unchanged");
	}

	#[test]
	fn advance_completely_blocked_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		// complete wall
		for row in 0..BOARD_HEIGHT {
			board[Coord { column: 2, row }] = Tile::Block;
		}

		let mut beast = SuperBeast::new(beast_position);
		let original_position = beast.position;
		let result = beast.advance(&mut board, player_position);

		assert_eq!(result, BeastAction::Moved, "Beast should return Moved");
		assert_ne!(beast.position, original_position, "Position should have changed going towards the player");
		assert_eq!(board[original_position], Tile::Empty, "Super SuperBeasts old position has been cleared");
		assert_eq!(board[beast.position], Tile::SuperBeast, "Super SuperBeasts new position has been set");
	}

	#[test]
	fn get_score_test() {
		assert_eq!(SuperBeast::get_score(), 6, "SuperBeast score should be 6");
	}
}
