use std::{cmp::Ordering, collections::HashMap};

use crate::{
	Coord, Dir, Tile,
	beasts::{Beast, BeastAction},
	board::Board,
	pathing::{get_end_of_block_chain, get_next_coord},
};

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	fn get_dir(from_position: Coord, to_position: Coord) -> Dir {
		match (to_position.column.cmp(&from_position.column), to_position.row.cmp(&from_position.row)) {
			(Ordering::Equal, Ordering::Greater) => {
				// current position is straight below
				Dir::Down
			},
			(Ordering::Equal, Ordering::Less) => {
				// current position is straight above
				Dir::Up
			},
			(Ordering::Less, Ordering::Equal) => {
				// current position is straight left
				Dir::Left
			},
			(Ordering::Greater, Ordering::Equal)
			| (Ordering::Greater, Ordering::Greater)
			| (Ordering::Greater, Ordering::Less)
			| (Ordering::Less, Ordering::Greater)
			| (Ordering::Less, Ordering::Less)
			| (Ordering::Equal, Ordering::Equal) => {
				// current position is straight right
				Dir::Right
			},
		}
	}

	fn is_diagonal(from_position: Coord, to_position: Coord) -> bool {
		from_position.column != to_position.column && from_position.row != to_position.row
	}

	fn astar_with_block_pushing(&self, board: &Board, player_position: Coord) -> Option<Vec<Coord>> {
		let start = self.position;
		let goal = player_position;

		let mut open_set = vec![start];
		let mut came_from: HashMap<Coord, Coord> = HashMap::new();

		let mut g_score: HashMap<Coord, i32> = HashMap::new();
		g_score.insert(start, 0);

		let mut f_score: HashMap<Coord, i32> = HashMap::new();
		f_score.insert(start, Self::heuristic(&start, &goal));

		while !open_set.is_empty() {
			let current = *open_set.iter().min_by_key(|coord| f_score.get(coord).unwrap_or(&i32::MAX)).unwrap();

			if current == goal {
				return Some(Self::reconstruct_path(&came_from, current));
			}

			open_set.retain(|&c| c != current);

			// generate neighbors, including those requiring block pushing
			for next_coord in Self::get_walkable_coords(board, &current, &player_position, false) {
				let mut valid_move = false;
				let mut squishes_player = false;

				match board[next_coord] {
					Tile::Empty | Tile::Player => {
						// direct movement to empty space or player
						valid_move = true;
					},
					Tile::Block => {
						// check if block can be pushed
						if !Self::is_diagonal(current, next_coord) {
							let dir = Self::get_dir(current, next_coord);
							if let Some((end_coord, _)) = get_end_of_block_chain(board, &next_coord, &dir) {
								match board[end_coord] {
									Tile::Empty => {
										// block can be pushed into empty space
										valid_move = true;
									},
									Tile::Player => {
										// block can be pushed to squish player
										valid_move = true;
										squishes_player = true;
									},
									_ => {
										// block can't be pushed (hits obstacle)
										valid_move = false;
									},
								}
							}
						} else {
							// block move on diagonals is not allowed
							valid_move = false;
						}
					},
					_ => {
						// not a valid move
						valid_move = false;
					},
				}

				if valid_move {
					let tentative_g_score = g_score.get(&current).unwrap_or(&i32::MAX) + 1;

					if tentative_g_score < *g_score.get(&next_coord).unwrap_or(&i32::MAX) {
						came_from.insert(next_coord, current);
						g_score.insert(next_coord, tentative_g_score);

						// if move squishes player, prioritize it by reducing heuristic
						let h_score = if squishes_player {
							Self::heuristic(&next_coord, &goal) - 10 // prioritize squishing
						} else {
							Self::heuristic(&next_coord, &goal)
						};

						f_score.insert(next_coord, tentative_g_score + h_score);

						if !open_set.contains(&next_coord) {
							open_set.push(next_coord);
						}
					}
				}
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
		// 1. check if player can be killed
		for next_coord in Self::get_walkable_coords(board, &self.position, &player_position, true) {
			if board[next_coord] == Tile::Player {
				board[self.position] = Tile::Empty;
				board[next_coord] = Tile::HatchedBeast;
				self.position = next_coord;
				return BeastAction::PlayerKilled;
			}
		}

		// 2. check if we can directly squish the player with a block
		for dir in [Dir::Up, Dir::Right, Dir::Down, Dir::Left] {
			if let Some(next_coord) = get_next_coord(&self.position, &dir) {
				if board[next_coord] == Tile::Block {
					if let Some((end_coord, _)) = get_end_of_block_chain(board, &next_coord, &dir) {
						if board[end_coord] == Tile::Player
							&& get_next_coord(&end_coord, &dir)
								.is_none_or(|coord| board[coord] == Tile::Block || board[coord] == Tile::StaticBlock)
						{
							board[self.position] = Tile::Empty;
							board[next_coord] = Tile::HatchedBeast;
							board[end_coord] = Tile::Block;
							self.position = next_coord;
							return BeastAction::PlayerKilled;
						}
					}
				}
			}
		}

		// 3. try to find a path using A* that considers block pushing
		if let Some(path) = self.astar_with_block_pushing(board, player_position) {
			if path.len() > 1 {
				// the first item is our own position
				let next_step = path[1];

				match board[next_step] {
					Tile::Player => {
						// this code path should not be hit since we check for it in step 1
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
					Tile::Block => {
						let dir = Self::get_dir(self.position, next_step);
						if let Some((end_coord, _)) = get_end_of_block_chain(board, &next_step, &dir) {
							match board[end_coord] {
								Tile::Empty => {
									board[self.position] = Tile::Empty;
									board[next_step] = Tile::HatchedBeast;
									board[end_coord] = Tile::Block;
									self.position = next_step;
									return BeastAction::Moved;
								},
								Tile::Player => {
									if get_next_coord(&end_coord, &dir)
										.is_none_or(|coord| board[coord] == Tile::Block || board[coord] == Tile::StaticBlock)
									{
										// this code path should also not be hit since we check for it in step 2
										board[self.position] = Tile::Empty;
										board[next_step] = Tile::HatchedBeast;
										board[end_coord] = Tile::Block;
										self.position = next_step;
										return BeastAction::PlayerKilled;
									}
								},
								_ => {},
							}
						}
					},
					_ => {
						// MAYBE: squish other beasts? I don't like the idea of it right now
					},
				}
			}
		}

		// 4. when there is no path we at least still go towards the player
		for neighbor in Self::get_walkable_coords(board, &self.position, &player_position, true) {
			match board[neighbor] {
				Tile::Player => {
					// this code path should not be hit since we check for it in step 1
					board[neighbor] = Tile::HatchedBeast;
					board[self.position] = Tile::Empty;
					self.position = neighbor;
					return BeastAction::PlayerKilled;
				},
				Tile::Empty => {
					board[neighbor] = Tile::HatchedBeast;
					board[self.position] = Tile::Empty;
					self.position = neighbor;
					return BeastAction::Moved;
				},
				_ => {},
			}
		}

		BeastAction::Stayed
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
	fn get_dir_test() {
		assert_eq!(
			HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 6 }),
			Dir::Down,
			"We are headed down"
		);
		assert_eq!(
			HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 6, row: 5 }),
			Dir::Right,
			"We are headed right"
		);
		assert_eq!(
			HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 4 }),
			Dir::Up,
			"We are headed up"
		);
		assert_eq!(
			HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 4, row: 5 }),
			Dir::Left,
			"We are headed left"
		);
	}

	#[test]
	fn advance_squish_straight_below_test() {
		// 5 ╬╬
		// 6 ░░
		// 7 ◀▶
		// 8 ░░
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 5, row: 7 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 8 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 5, row: 6 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 5, row: 7 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 8 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);

		// 5 ╬╬
		// 6 ░░
		// 7 ░░
		// 8 ◀▶
		// 9 ░░
		let beast_position = Coord { column: 5, row: 5 };
		beast.position = beast_position;

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 7 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 9 }] = Tile::Block;

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 5, row: 6 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 5, row: 7 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 8 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 9 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
	}

	#[test]
	fn advance_squish_straight_above_test() {
		// 2 ░░
		// 3 ◀▶
		// 4 ░░
		// 5 ╬╬
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 5, row: 3 };

		board[Coord { column: 5, row: 2 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let mut beast = HatchedBeast::new(beast_position);
		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 5, row: 4 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 5, row: 3 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 2 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);

		// 2 ░░
		// 3 ◀▶
		// 4 ░░
		// 5 ░░
		// 6 ╬╬
		let beast_position = Coord { column: 5, row: 6 };
		beast.position = beast_position;

		board[Coord { column: 5, row: 2 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 5, row: 5 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 5, row: 4 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 3 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 5, row: 2 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
	}

	#[test]
	fn advance_squish_straight_left_test() {
		// 5 ░░◀▶░░╬╬
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 3, row: 5 };

		board[Coord { column: 2, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let mut beast = HatchedBeast::new(beast_position);
		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 4, row: 5 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 3, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 2, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);

		// 5 ░░◀▶░░░░╬╬
		let beast_position = Coord { column: 6, row: 5 };
		beast.position = beast_position;

		board[Coord { column: 2, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 5, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 5, row: 5 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 4, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 3, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 2, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
	}

	#[test]
	fn advance_squish_straight_right_test() {
		// 5 ╬╬░░◀▶░░
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 7, row: 5 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 8, row: 5 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 6, row: 5 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 7, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 8, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);

		// 5 ╬╬░░░░◀▶░░
		let player_position = Coord { column: 8, row: 5 };
		beast.position = beast_position;

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 7, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 9, row: 5 }] = Tile::Block;

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast killed the player");
		assert_eq!(beast.position, Coord { column: 6, row: 5 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board[Coord { column: 7, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 8, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
		assert_eq!(
			board[Coord { column: 9, row: 5 }],
			Tile::Block,
			"The block tile is in the correct position on the board"
		);
	}

	#[test]
	fn advance_blocked_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░      ░░
		// 1 ▌  ◀▶  ░░  ╬╬  ░░
		// 2 ▌      ░░░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 1 };
		let beast_position = Coord { column: 5, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;
		board[Coord { column: 4, row: 2 }] = Tile::Block;
		board[Coord { column: 5, row: 2 }] = Tile::Block;
		board[Coord { column: 6, row: 2 }] = Tile::Block;
		board[Coord { column: 7, row: 2 }] = Tile::Block;
		board[Coord { column: 7, row: 1 }] = Tile::Block;
		board[Coord { column: 7, row: 0 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░      ░░
		// 1 ▌  ◀▶  ░░╬╬    ░░
		// 2 ▌      ░░░░░░░░░░
		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 4, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			9,
			"There should be exactly nine block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░      ░░
		// 1 ▌  ◀▶░░╬╬      ░░
		// 2 ▌      ░░░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 3, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::Block, "The block has moved");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			9,
			"There should be exactly nine block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ╬╬░░      ░░
		// 1 ▌  ◀▶░░        ░░
		// 2 ▌      ░░░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 2, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			9,
			"There should be exactly nine block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░      ░░
		// 1 ▌  ╬╬░░        ░░
		// 2 ▌      ░░░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast has killed the player");
		assert_eq!(beast.position, Coord { column: 1, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			9,
			"There should be exactly nine block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
	}

	#[test]
	fn advance_blockchain_squish_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌◀▶    ░░░░░░  ╬╬
		// 2 ▌░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 1 };
		let beast_position = Coord { column: 7, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 4, row: 1 }] = Tile::Block;
		board[Coord { column: 5, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌◀▶    ░░░░░░╬╬
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 6, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			8,
			"There should be exactly eight block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌◀▶  ░░░░░░╬╬
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 5, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			8,
			"There should be exactly eight block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌◀▶░░░░░░╬╬
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 4, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			8,
			"There should be exactly eight block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌░░░░░░╬╬
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast has killed the player");
		assert_eq!(beast.position, Coord { column: 3, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			8,
			"There should be exactly eight block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
	}

	#[test]
	fn advance_blockchain_staticblock_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ◀▶  ░░▓▓░░  ╬╬
		// 2 ▌░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 1 };
		let beast_position = Coord { column: 7, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 4, row: 1 }] = Tile::StaticBlock;
		board[Coord { column: 5, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ◀▶  ░░▓▓░░╬╬
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 6, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░  ╬╬
		// 1 ▌  ◀▶  ░░▓▓░░
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 5, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░╬╬
		// 1 ▌  ◀▶  ░░▓▓░░
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 4, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░╬╬
		// 1 ▌  ◀▶  ░░▓▓░░
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 3, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌  ◀▶╬╬░░▓▓░░
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 2, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(board[player_position], Tile::Player, "The player hasn't moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌  ╬╬  ░░▓▓░░
		// 2 ▌░░░░░░░░

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast has killed the player");
		assert_eq!(beast.position, Coord { column: 1, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			7,
			"There should be exactly seven block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
	}

	#[test]
	fn advance_squishable_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌▓▓◀▶░░╬╬

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 0 };
		let beast_position = Coord { column: 3, row: 0 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 0, row: 0 }] = Tile::StaticBlock;
		board[Coord { column: 2, row: 0 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌▓▓░░╬╬

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::PlayerKilled, "The beast has killed the player");
		assert_eq!(beast.position, Coord { column: 2, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
	}

	#[test]
	fn advance_non_squishable_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌├┤◀▶░░╬╬
		// 1 ▌

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 0 };
		let beast_position = Coord { column: 3, row: 0 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 0, row: 0 }] = Tile::CommonBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌├┤◀▶░░
		// 1 ▌    ╬╬

		let beast_action = beast.advance(&mut board, player_position);
		assert_eq!(beast_action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 2, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			1,
			"There should be exactly one common beast tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
	}

	#[test]
	fn advance_a_star_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ▓▓
		// 1 ▌◀▶  ▓▓  ╬╬
		// 2 ▌    ▓▓
		// 3 ▌

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 1 };
		let beast_position = Coord { column: 4, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::StaticBlock;
		board[Coord { column: 2, row: 1 }] = Tile::StaticBlock;
		board[Coord { column: 2, row: 2 }] = Tile::StaticBlock;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ▓▓
		// 1 ▌◀▶  ▓▓
		// 2 ▌    ▓▓╬╬
		// 3 ▌

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 3, row: 2 }, "Beast moved to correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The new position shows a beast tile");
		assert_eq!(board[Coord { column: 3, row: 1 }], Tile::Empty, "The previous position should be empty");

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ▓▓
		// 1 ▌◀▶  ▓▓
		// 2 ▌    ▓▓
		// 3 ▌    ╬╬

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 2, row: 3 }, "Beast moved to correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The new position shows a beast tile");
		assert_eq!(board[Coord { column: 3, row: 3 }], Tile::Empty, "The previous position should be empty");

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ▓▓
		// 1 ▌◀▶  ▓▓
		// 2 ▌  ╬╬▓▓
		// 3 ▌

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast has moved");
		assert_eq!(beast.position, Coord { column: 1, row: 2 }, "Beast moved to correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The new position shows a beast tile");
		assert_eq!(board[Coord { column: 1, row: 3 }], Tile::Empty, "The previous position should be empty");

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ▓▓
		// 1 ▌╬╬  ▓▓
		// 2 ▌    ▓▓
		// 3 ▌

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::PlayerKilled, "The beast has killed the player");
		assert_eq!(beast.position, Coord { column: 0, row: 1 }, "Beast moved to correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The new position shows a beast tile");
		assert_eq!(board[Coord { column: 1, row: 2 }], Tile::Empty, "The previous position should be empty");
	}

	#[test]
	fn advance_blocked_via_player_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌  ◀▶░░╬╬
		// 2 ▌░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 1 };
		let beast_position = Coord { column: 3, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░╬╬
		// 1 ▌  ◀▶░░
		// 2 ▌░░░░░░

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast should move");
		assert_eq!(beast.position, Coord { column: 3, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌  ░░╬╬
		// 1 ▌  ◀▶░░
		// 2 ▌░░░░░░

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast should move");
		assert_eq!(beast.position, Coord { column: 2, row: 0 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌  ░░
		// 1 ▌  ╬╬░░
		// 2 ▌░░░░░░

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::PlayerKilled, "The beast should killed the player");
		assert_eq!(beast.position, Coord { column: 1, row: 1 }, "The beast moved to the correct position");
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
	}

	#[test]
	fn advance_blocked_board_end_test() {
		//    0 1 2 3 4 5 6 7
		// 26 ▌    ╬╬
		// 27 ▌░░░░░░
		// 28 ▌  ◀▶░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord {
			column: 1,
			row: BOARD_HEIGHT - 2,
		};
		let beast_position = Coord {
			column: 2,
			row: BOARD_HEIGHT - 4,
		};

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord {
			column: 0,
			row: BOARD_HEIGHT - 3,
		}] = Tile::Block;
		board[Coord {
			column: 1,
			row: BOARD_HEIGHT - 3,
		}] = Tile::Block;
		board[Coord {
			column: 2,
			row: BOARD_HEIGHT - 3,
		}] = Tile::Block;
		board[Coord {
			column: 2,
			row: BOARD_HEIGHT - 2,
		}] = Tile::Block;
		board[Coord {
			column: 2,
			row: BOARD_HEIGHT - 1,
		}] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		//    0 1 2 3 4 5 6 7
		// 26 ▌  ╬╬
		// 27 ▌░░░░░░
		// 28 ▌  ◀▶░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast should move");
		assert_eq!(
			beast.position,
			Coord {
				column: 1,
				row: BOARD_HEIGHT - 4
			},
			"The beast moved to the correct position"
		);
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");

		//    0 1 2 3 4 5 6 7
		// 26 ▌╬╬
		// 27 ▌░░░░░░
		// 28 ▌  ◀▶░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast should move");
		assert_eq!(
			beast.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 4
			},
			"The beast moved to the correct position"
		);
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");

		//    0 1 2 3 4 5 6 7
		// 26 ▌
		// 27 ▌╬╬░░░░
		// 28 ▌░░◀▶░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::Moved, "The beast should move");
		assert_eq!(
			beast.position,
			Coord {
				column: 0,
				row: BOARD_HEIGHT - 3
			},
			"The beast moved to the correct position"
		);
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");

		//    0 1 2 3 4 5 6 7
		// 26 ▌
		// 27 ▌  ░░░░
		// 28 ▌░░╬╬░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let action = beast.advance(&mut board, player_position);
		assert_eq!(action, BeastAction::PlayerKilled, "The beast should have killed the player");
		assert_eq!(
			beast.position,
			Coord {
				column: 1,
				row: BOARD_HEIGHT - 2
			},
			"The beast moved to the correct position"
		);
		assert_eq!(board[beast.position], Tile::HatchedBeast, "The beast tile is in the correct position on the board");
	}
}
