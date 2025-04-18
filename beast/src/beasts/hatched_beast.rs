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
			for dir in [Dir::Up, Dir::Right, Dir::Down, Dir::Left] {
				if let Some(next_coord) = get_next_coord(&current, &dir) {
					let mut valid_move = false;
					let mut squishes_player = false;

					match board[next_coord] {
						Tile::Empty | Tile::Player => {
							// direct movement to empty space or player
							valid_move = true;
						},
						Tile::Block => {
							// check if block can be pushed
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
		}

		None
	}
}

impl Beast for HatchedBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		// 1. check if we can directly squish the player with a block
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

		// 2. try to find a path using A* that considers block pushing
		if let Some(path) = self.astar_with_block_pushing(board, player_position) {
			if path.len() > 1 {
				// the first item is our own position
				let next_step = path[1];
				let dir = Self::get_dir(self.position, next_step);

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
					Tile::Block => {
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
						// TODO: squish other beasts?
					},
				}
			}
		}

		// 3. when there is no path we at least still go towards the player
		for neighbor in Self::get_walkable_coords(board, &self.position, &player_position, true) {
			match board[neighbor] {
				Tile::Player => {
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

		BeastAction::Moved
	}

	fn get_score() -> u16 {
		2
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	// use crate::{BOARD_HEIGHT, BOARD_WIDTH};
	// TODO

	#[test]
	fn get_dir_test() {
		assert_eq!(HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 6 }), Dir::Down);
		assert_eq!(HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 6, row: 5 }), Dir::Right);
		assert_eq!(HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 4 }), Dir::Up);
		assert_eq!(HatchedBeast::get_dir(Coord { column: 5, row: 5 }, Coord { column: 4, row: 5 }), Dir::Left);
	}

	// #[test]
	// fn try_squish_player_straight_below_test() {
	// 	// 5 ╬╬
	// 	// 6 ░░
	// 	// 7 ◀▶
	// 	// 8 ░░
	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let beast_position = Coord { column: 5, row: 5 };
	// 	let player_position = Coord { column: 5, row: 7 };

	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 5, row: 6 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 5, row: 8 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 6 }, player_position)));

	// 	// 5 ╬╬
	// 	// 6 ░░
	// 	// 7 ░░
	// 	// 8 ◀▶
	// 	// 9 ░░
	// 	let player_position = Coord { column: 5, row: 8 };

	// 	board[Coord { column: 5, row: 7 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 5, row: 9 }] = Tile::Block;

	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 6 }, player_position)));
	// }

	// #[test]
	// fn try_squish_player_straight_above_test() {
	// 	// 2 ░░
	// 	// 3 ◀▶
	// 	// 4 ░░
	// 	// 5 ╬╬
	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let beast_position = Coord { column: 5, row: 5 };
	// 	let player_position = Coord { column: 5, row: 3 };

	// 	board[Coord { column: 5, row: 2 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 5, row: 4 }] = Tile::Block;
	// 	board[beast_position] = Tile::HatchedBeast;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 4 }, player_position)));

	// 	// 2 ░░
	// 	// 3 ◀▶
	// 	// 4 ░░
	// 	// 5 ░░
	// 	// 6 ╬╬
	// 	let beast_position = Coord { column: 5, row: 6 };

	// 	board[Coord { column: 5, row: 5 }] = Tile::Block;
	// 	board[beast_position] = Tile::HatchedBeast;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 5 }, player_position)));
	// }

	// #[test]
	// fn try_squish_player_straight_left_test() {
	// 	// 5 ░░◀▶░░╬╬
	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let beast_position = Coord { column: 5, row: 5 };
	// 	let player_position = Coord { column: 3, row: 5 };

	// 	board[Coord { column: 2, row: 5 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 4, row: 5 }] = Tile::Block;
	// 	board[beast_position] = Tile::HatchedBeast;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 4, row: 5 }, player_position)));

	// 	// 5 ░░◀▶░░░░╬╬
	// 	let beast_position = Coord { column: 6, row: 5 };

	// 	board[Coord { column: 5, row: 5 }] = Tile::Block;
	// 	board[beast_position] = Tile::HatchedBeast;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 5 }, player_position)));
	// }

	// #[test]
	// fn try_squish_player_straight_right_test() {
	// 	// 5 ╬╬░░◀▶░░
	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let beast_position = Coord { column: 5, row: 5 };
	// 	let player_position = Coord { column: 7, row: 5 };

	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 6, row: 5 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 8, row: 5 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 6, row: 5 }, player_position)));

	// 	// 5 ╬╬░░░░◀▶░░
	// 	let player_position = Coord { column: 8, row: 5 };

	// 	board[Coord { column: 7, row: 5 }] = Tile::Block;
	// 	board[player_position] = Tile::Player;
	// 	board[Coord { column: 9, row: 5 }] = Tile::Block;

	// 	assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 6, row: 5 }, player_position)));
	// }

	// #[test]
	// fn try_find_movable_block_left_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░      ░░
	// 	// 1 ▌  ◀▶  ░░  ╬╬  ░░
	// 	// 2 ▌      ░░░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 1, row: 1 };
	// 	let beast_position = Coord { column: 5, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 6, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 7, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 7, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 7, row: 0 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));
	// }

	// #[test]
	// fn try_find_movable_block_right_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌  ░░      ░░
	// 	// 1 ▌  ░░  ╬╬  ░░  ◀▶
	// 	// 2 ▌  ░░░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 7, row: 1 };
	// 	let beast_position = Coord { column: 3, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 1, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 0 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 5, row: 1 }));
	// }

	// #[test]
	// fn try_find_movable_block_top_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ◀▶
	// 	// 1 ▌
	// 	// 2 ▌  ░░░░░░░░░░
	// 	// 3 ▌  ░░  ╬╬  ░░
	// 	// 4 ▌  ░░░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 3, row: 0 };
	// 	let beast_position = Coord { column: 3, row: 3 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 4 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 4 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 4 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 4 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 4 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 2 }));
	// }

	// #[test]
	// fn try_find_movable_block_bottom_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌  ░░  ╬╬  ░░
	// 	// 1 ▌  ░░░░░░░░░░
	// 	// 2 ▌
	// 	// 3 ▌      ◀▶

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 3, row: 3 };
	// 	let beast_position = Coord { column: 3, row: 0 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 1, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 1 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));
	// }

	// #[test]
	// fn try_find_movable_block_blockchain_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░░░░░  ◀▶
	// 	// 2 ▌░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 7, row: 1 };
	// 	let beast_position = Coord { column: 1, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░░░▓▓  ◀▶
	// 	// 2 ▌░░░░░░░░
	// 	board[Coord { column: 5, row: 1 }] = Tile::StaticBlock;
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	// }

	// #[test]
	// fn try_find_movable_block_squishable_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░◀▶▓▓
	// 	// 2 ▌░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 4, row: 1 };
	// 	let beast_position = Coord { column: 1, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 1 }] = Tile::StaticBlock;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░◀▶├┤
	// 	// 2 ▌░░░░░░░░
	// 	board[Coord { column: 5, row: 1 }] = Tile::CommonBeast;
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	// }

	// #[test]
	// fn try_find_movable_block_blockchain_squishable_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░░░░░◀▶▓▓
	// 	// 2 ▌░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 6, row: 1 };
	// 	let beast_position = Coord { column: 1, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 5, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 7, row: 1 }] = Tile::StaticBlock;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░░░░░◀▶╟╢
	// 	// 2 ▌░░░░░░░░
	// 	board[Coord { column: 7, row: 1 }] = Tile::SuperBeast;
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	// }

	// #[test]
	// fn try_find_movable_block_nopath_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░░░
	// 	// 1 ▌  ╬╬  ░░░░  ◀▶
	// 	// 2 ▌░░░░░░░░░░
	// 	// 3 ▌░░░░░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 6, row: 1 };
	// 	let beast_position = Coord { column: 1, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 4, row: 3 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	// }

	// #[test]
	// fn try_find_movable_block_diagonal_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░
	// 	// 2 ▌      ░░
	// 	// 3 ▌░░░░░░░░
	// 	// 4 ▌            ◀▶

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 5, row: 4 };
	// 	let beast_position = Coord { column: 1, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 3, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 3 }] = Tile::Block;
	// 	board[Coord { column: 3, row: 3 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

	// 	//    0 1 2 3 4 5 6 7 8
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌      ░░
	// 	// 1 ▌  ╬╬  ░░
	// 	// 2 ▌      ░░
	// 	// 3 ▌░░░░░░░░
	// 	// 4 ▌
	// 	// 5 ▌
	// 	// 6 ▌    ◀▶
	// 	board[player_position] = Tile::Empty;
	// 	let player_position = Coord { column: 2, row: 6 };
	// 	board[player_position] = Tile::Player;

	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 1, row: 3 }));
	// }

	// #[test]
	// fn try_find_movable_block_push_against_frame_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌  ◀▶░░
	// 	// 2 ▌░░░░░░  ╬╬

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 1, row: 1 };
	// 	let beast_position = Coord { column: 4, row: 2 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	// }

	// #[test]
	// fn try_find_movable_block_push_to_open_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░  ╬╬
	// 	// 2 ▌░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 0, row: 1 };
	// 	let beast_position = Coord { column: 4, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;

	// 	let beast = HatchedBeast::new(beast_position);
	// 	assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 2, row: 1 }));
	// }

	// #[test]
	// fn advance_squish_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌░░◀▶░░╬╬

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 1, row: 0 };
	// 	let beast_position = Coord { column: 3, row: 0 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 0, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;

	// 	let mut beast = HatchedBeast::new(beast_position);
	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌░░░░╬╬

	// 	assert_eq!(action, BeastAction::PlayerKilled);
	// 	assert_eq!(beast.position, Coord { column: 2, row: 0 });
	// 	assert_eq!(board[Coord { column: 3, row: 0 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 2, row: 0 }], Tile::HatchedBeast);
	// }

	// #[test]
	// fn advance_a_star_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░  ╬╬
	// 	// 2 ▌    ░░
	// 	// 3 ▌

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 0, row: 1 };
	// 	let beast_position = Coord { column: 4, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;

	// 	let mut beast = HatchedBeast::new(beast_position);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░
	// 	// 2 ▌    ░░╬╬
	// 	// 3 ▌

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 3, row: 2 });
	// 	assert_eq!(board[Coord { column: 4, row: 1 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 3, row: 2 }], Tile::HatchedBeast);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░
	// 	// 2 ▌    ░░
	// 	// 3 ▌    ╬╬

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 2, row: 3 });
	// 	assert_eq!(board[Coord { column: 3, row: 2 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 2, row: 3 }], Tile::HatchedBeast);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░
	// 	// 2 ▌  ╬╬░░
	// 	// 3 ▌

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 1, row: 2 });
	// 	assert_eq!(board[Coord { column: 2, row: 3 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 1, row: 2 }], Tile::HatchedBeast);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌╬╬  ░░
	// 	// 2 ▌    ░░
	// 	// 3 ▌

	// 	assert_eq!(action, BeastAction::PlayerKilled);
	// 	assert_eq!(beast.position, Coord { column: 0, row: 1 });
	// 	assert_eq!(board[Coord { column: 1, row: 2 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 0, row: 1 }], Tile::HatchedBeast);
	// }

	// #[test]
	// fn advance_blocked_squish_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░  ╬╬
	// 	// 2 ▌░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 0, row: 1 };
	// 	let beast_position = Coord { column: 4, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;

	// 	let mut beast = HatchedBeast::new(beast_position);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶  ░░╬╬
	// 	// 2 ▌░░░░░░

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 3, row: 1 });
	// 	assert_eq!(board[Coord { column: 4, row: 1 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 3, row: 1 }], Tile::HatchedBeast);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌◀▶░░╬╬
	// 	// 2 ▌░░░░░░

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 2, row: 1 });
	// 	assert_eq!(board[Coord { column: 3, row: 1 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 2, row: 1 }], Tile::HatchedBeast);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌░░╬╬
	// 	// 2 ▌░░░░░░

	// 	assert_eq!(action, BeastAction::PlayerKilled);
	// 	assert_eq!(beast.position, Coord { column: 1, row: 1 });
	// 	assert_eq!(board[Coord { column: 2, row: 1 }], Tile::Empty);
	// 	assert_eq!(board[Coord { column: 1, row: 1 }], Tile::HatchedBeast);
	// }

	// #[test]
	// fn advance_blocked_via_player_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌  ◀▶░░╬╬
	// 	// 2 ▌░░░░░░

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 1, row: 1 };
	// 	let beast_position = Coord { column: 3, row: 1 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 2, row: 0 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 1 }] = Tile::Block;
	// 	board[Coord { column: 0, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 2 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 2 }] = Tile::Block;

	// 	let mut beast = HatchedBeast::new(beast_position);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
	// 	// 0 ▌    ░░
	// 	// 1 ▌  ◀▶░░╬╬
	// 	// 2 ▌░░░░░░

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 3, row: 1 });
	// 	assert_eq!(board[Coord { column: 3, row: 1 }], Tile::HatchedBeast);
	// }

	// #[test]
	// fn advance_blocked_diagonal_next_test() {
	// 	//    0 1 2 3 4 5 6 7
	// 	// 26 ▌    ╬╬
	// 	// 27 ▌░░░░░░
	// 	// 28 ▌◀▶  ░░
	// 	// 29 ▌    ░░
	// 	//    ▙▄▄▄▄▄▄

	// 	let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
	// 	let player_position = Coord { column: 0, row: 28 };
	// 	let beast_position = Coord { column: 2, row: 26 };

	// 	board[player_position] = Tile::Player;
	// 	board[beast_position] = Tile::HatchedBeast;
	// 	board[Coord { column: 0, row: 27 }] = Tile::Block;
	// 	board[Coord { column: 1, row: 27 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 27 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 28 }] = Tile::Block;
	// 	board[Coord { column: 2, row: 29 }] = Tile::Block;

	// 	let mut beast = HatchedBeast::new(beast_position);

	// 	let action = beast.advance(&mut board, player_position);
	// 	//    0 1 2 3 4 5 6 7
	// 	// 26 ▌    ╬╬
	// 	// 27 ▌░░░░░░
	// 	// 28 ▌◀▶  ░░
	// 	// 29 ▌    ░░
	// 	//    ▙▄▄▄▄▄▄

	// 	assert_eq!(action, BeastAction::Moved);
	// 	assert_eq!(beast.position, Coord { column: 2, row: 26 });
	// 	assert_eq!(board[Coord { column: 2, row: 26 }], Tile::HatchedBeast);
	// }
}
