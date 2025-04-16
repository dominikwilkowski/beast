use crate::{
	Coord, Dir, Tile,
	beasts::{Beast, BeastAction},
	board::Board,
	pathing::{get_dir, get_next_coord},
};

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	fn try_squish_player(&self, board: &Board, player_position: Coord) -> Option<(Coord, Coord)> {
		let dir = get_dir(self.position, player_position);

		let mut next_tile = Tile::Block;
		let mut prev_coord = self.position;
		let mut block_position = None;

		while next_tile == Tile::Block {
			if let Some(next_coord) = get_next_coord(prev_coord, &dir) {
				next_tile = board[next_coord];

				match next_tile {
					Tile::Block => {
						if block_position.is_none() {
							block_position = Some(next_coord);
						}
					},
					Tile::Player => {
						if let Some(block) = block_position {
							if get_next_coord(next_coord, &dir)
								.is_none_or(|coord| board[coord] == Tile::Block || board[coord] == Tile::StaticBlock)
							{
								return Some((block, next_coord));
							}
						}
					},
					_ => {
						return None;
					},
				}

				prev_coord = next_coord;
			} else {
				break;
			}
		}

		None
	}

	fn try_find_movable_block(&self, board: &Board, player_position: Coord) -> Option<Coord> {
		let mut current_position = self.position;
		let mut dir = Dir::Right;
		while player_position != current_position {
			if board[current_position] == Tile::Block {
				let block_position = current_position;
				let mut temp_board = *board;

				if let Some(next_tile) = get_next_coord(current_position, &dir) {
					match temp_board[next_tile] {
						Tile::Empty => {
							temp_board[block_position] = Tile::Empty;
							temp_board[next_tile] = Tile::Block;
						},
						Tile::Block => {
							let mut next_block_tile = next_tile;
							while temp_board[next_block_tile] == Tile::Block {
								if let Some(next_coord) = get_next_coord(next_block_tile, &dir) {
									match temp_board[next_coord] {
										Tile::Block => {
											next_block_tile = next_coord;
										},
										Tile::Empty => {
											temp_board[block_position] = Tile::Empty;
											temp_board[next_coord] = Tile::Block;
											break;
										},
										Tile::Player => {
											if get_next_coord(next_coord, &dir)
												.is_none_or(|coord| temp_board[coord] == Tile::Block || temp_board[coord] == Tile::StaticBlock)
											{
												// there is a chain of blocks (a blockchain) that can be pushed to squish the player
												return Some(block_position);
											} else {
												break;
											}
										},
										_ => {
											break;
										},
									}
								} else {
									// we are pushing against the frame of the board
									break;
								}
							}
						},
						Tile::Player => {
							if get_next_coord(next_tile, &dir)
								.is_none_or(|coord| temp_board[coord] == Tile::Block || temp_board[coord] == Tile::StaticBlock)
							{
								// there is a chain of blocks (a blockchain) that can be pushed to squish the player
								return Some(block_position);
							}
						},
						_ => {},
					}
				}

				if let Some(path) = Self::astar(&temp_board, self.position, &player_position) {
					if path.len() > 1 {
						return Some(current_position);
					}
				}
			}

			let column_diff = player_position.column as isize - current_position.column as isize;
			let row_diff = player_position.row as isize - current_position.row as isize;

			if column_diff.abs() >= row_diff.abs() && column_diff != 0 {
				let step = column_diff.signum();
				current_position = Coord {
					column: (current_position.column as isize + step) as usize,
					row: current_position.row,
				};
				dir = if step > 0 { Dir::Right } else { Dir::Left };
			} else {
				let step = row_diff.signum();
				current_position = Coord {
					column: current_position.column,
					row: (current_position.row as isize + step) as usize,
				};
				dir = if step > 0 { Dir::Down } else { Dir::Up };
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
		if let Some(movable_block) = self.try_find_movable_block(board, player_position) {
			let column_diff = self.position.column as isize - movable_block.column as isize;
			let row_diff = self.position.row as isize - movable_block.row as isize;
			if column_diff.abs() + row_diff.abs() == 1 {
				// if the block is right next to us
				let dir = get_dir(self.position, movable_block);

				let mut next_tile = Tile::Block;
				let mut prev_coord = movable_block;

				while next_tile == Tile::Block {
					if let Some(next_coord) = get_next_coord(prev_coord, &dir) {
						next_tile = board[next_coord];

						match next_tile {
							Tile::Block => {},
							Tile::Player => {
								if get_next_coord(next_coord, &dir)
									.is_none_or(|coord| board[coord] == Tile::Block || board[coord] == Tile::StaticBlock)
								{
									board[movable_block] = Tile::HatchedBeast;
									board[self.position] = Tile::Empty;
									self.position = movable_block;
									board[next_coord] = Tile::Block;
									return BeastAction::PlayerKilled;
								}
							},
							Tile::Empty => {
								board[movable_block] = Tile::HatchedBeast;
								board[self.position] = Tile::Empty;
								self.position = movable_block;
								board[next_coord] = Tile::Block;
								return BeastAction::Moved;
							},
							_ => {},
						}

						prev_coord = next_coord;
					}
				}
			} else {
				// the block is not right next to us so we have to path find our way there
				let mut temp_board = *board;
				temp_board[movable_block] = Tile::Player;
				if let Some(path) = Self::astar(&temp_board, self.position, &movable_block) {
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
			}
		}

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
	use crate::{BOARD_HEIGHT, BOARD_WIDTH};

	#[test]
	fn try_squish_player_straight_below_test() {
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

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 6 }, player_position)));

		// 5 ╬╬
		// 6 ░░
		// 7 ░░
		// 8 ◀▶
		// 9 ░░
		let player_position = Coord { column: 5, row: 8 };

		board[Coord { column: 5, row: 7 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 5, row: 9 }] = Tile::Block;

		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 6 }, player_position)));
	}

	#[test]
	fn try_squish_player_straight_above_test() {
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

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 4 }, player_position)));

		// 2 ░░
		// 3 ◀▶
		// 4 ░░
		// 5 ░░
		// 6 ╬╬
		let beast_position = Coord { column: 5, row: 6 };

		board[Coord { column: 5, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 5 }, player_position)));
	}

	#[test]
	fn try_squish_player_straight_left_test() {
		// 5 ░░◀▶░░╬╬
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 3, row: 5 };

		board[Coord { column: 2, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 4, row: 5 }, player_position)));

		// 5 ░░◀▶░░░░╬╬
		let beast_position = Coord { column: 6, row: 5 };

		board[Coord { column: 5, row: 5 }] = Tile::Block;
		board[beast_position] = Tile::HatchedBeast;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 5, row: 5 }, player_position)));
	}

	#[test]
	fn try_squish_player_straight_right_test() {
		// 5 ╬╬░░◀▶░░
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 5, row: 5 };
		let player_position = Coord { column: 7, row: 5 };

		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 8, row: 5 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 6, row: 5 }, player_position)));

		// 5 ╬╬░░░░◀▶░░
		let player_position = Coord { column: 8, row: 5 };

		board[Coord { column: 7, row: 5 }] = Tile::Block;
		board[player_position] = Tile::Player;
		board[Coord { column: 9, row: 5 }] = Tile::Block;

		assert_eq!(beast.try_squish_player(&board, player_position), Some((Coord { column: 6, row: 5 }, player_position)));
	}

	#[test]
	fn try_find_movable_block_left_test() {
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

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));
	}

	#[test]
	fn try_find_movable_block_right_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌  ░░      ░░
		// 1 ▌  ░░  ╬╬  ░░  ◀▶
		// 2 ▌  ░░░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 7, row: 1 };
		let beast_position = Coord { column: 3, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 1, row: 0 }] = Tile::Block;
		board[Coord { column: 1, row: 1 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;
		board[Coord { column: 4, row: 2 }] = Tile::Block;
		board[Coord { column: 5, row: 2 }] = Tile::Block;
		board[Coord { column: 5, row: 1 }] = Tile::Block;
		board[Coord { column: 5, row: 0 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 5, row: 1 }));
	}

	#[test]
	fn try_find_movable_block_top_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ◀▶
		// 1 ▌
		// 2 ▌  ░░░░░░░░░░
		// 3 ▌  ░░  ╬╬  ░░
		// 4 ▌  ░░░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 3, row: 0 };
		let beast_position = Coord { column: 3, row: 3 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;
		board[Coord { column: 4, row: 2 }] = Tile::Block;
		board[Coord { column: 5, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 3 }] = Tile::Block;
		board[Coord { column: 5, row: 3 }] = Tile::Block;
		board[Coord { column: 1, row: 4 }] = Tile::Block;
		board[Coord { column: 2, row: 4 }] = Tile::Block;
		board[Coord { column: 3, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 2 }));
	}

	#[test]
	fn try_find_movable_block_bottom_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌  ░░  ╬╬  ░░
		// 1 ▌  ░░░░░░░░░░
		// 2 ▌
		// 3 ▌      ◀▶

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 3, row: 3 };
		let beast_position = Coord { column: 3, row: 0 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 1, row: 0 }] = Tile::Block;
		board[Coord { column: 5, row: 0 }] = Tile::Block;
		board[Coord { column: 1, row: 1 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 4, row: 1 }] = Tile::Block;
		board[Coord { column: 5, row: 1 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));
	}

	#[test]
	fn try_find_movable_block_blockchain_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░░░░░  ◀▶
		// 2 ▌░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 7, row: 1 };
		let beast_position = Coord { column: 1, row: 1 };

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

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░░░▓▓  ◀▶
		// 2 ▌░░░░░░░░
		board[Coord { column: 5, row: 1 }] = Tile::StaticBlock;
		assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	}

	#[test]
	fn try_find_movable_block_squishable_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░◀▶▓▓
		// 2 ▌░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 4, row: 1 };
		let beast_position = Coord { column: 1, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 5, row: 1 }] = Tile::StaticBlock;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░◀▶├┤
		// 2 ▌░░░░░░░░
		board[Coord { column: 5, row: 1 }] = Tile::CommonBeast;
		assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	}

	#[test]
	fn try_find_movable_block_blockchain_squishable_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░░░░░◀▶▓▓
		// 2 ▌░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 6, row: 1 };
		let beast_position = Coord { column: 1, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 4, row: 1 }] = Tile::Block;
		board[Coord { column: 5, row: 1 }] = Tile::Block;
		board[Coord { column: 7, row: 1 }] = Tile::StaticBlock;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░░░░░◀▶╟╢
		// 2 ▌░░░░░░░░
		board[Coord { column: 7, row: 1 }] = Tile::SuperBeast;
		assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	}

	#[test]
	fn try_find_movable_block_nopath_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░░░
		// 1 ▌  ╬╬  ░░░░  ◀▶
		// 2 ▌░░░░░░░░░░
		// 3 ▌░░░░░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 6, row: 1 };
		let beast_position = Coord { column: 1, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 4, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 4, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;
		board[Coord { column: 4, row: 2 }] = Tile::Block;
		board[Coord { column: 0, row: 3 }] = Tile::Block;
		board[Coord { column: 1, row: 3 }] = Tile::Block;
		board[Coord { column: 2, row: 3 }] = Tile::Block;
		board[Coord { column: 3, row: 3 }] = Tile::Block;
		board[Coord { column: 4, row: 3 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	}

	#[test]
	fn try_find_movable_block_diagonal_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░
		// 2 ▌      ░░
		// 3 ▌░░░░░░░░
		// 4 ▌            ◀▶

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 5, row: 4 };
		let beast_position = Coord { column: 1, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 3, row: 0 }] = Tile::Block;
		board[Coord { column: 3, row: 1 }] = Tile::Block;
		board[Coord { column: 3, row: 2 }] = Tile::Block;
		board[Coord { column: 0, row: 3 }] = Tile::Block;
		board[Coord { column: 1, row: 3 }] = Tile::Block;
		board[Coord { column: 2, row: 3 }] = Tile::Block;
		board[Coord { column: 3, row: 3 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 3, row: 1 }));

		//    0 1 2 3 4 5 6 7 8
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌      ░░
		// 1 ▌  ╬╬  ░░
		// 2 ▌      ░░
		// 3 ▌░░░░░░░░
		// 4 ▌
		// 5 ▌
		// 6 ▌    ◀▶
		board[player_position] = Tile::Empty;
		let player_position = Coord { column: 2, row: 6 };
		board[player_position] = Tile::Player;

		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 1, row: 3 }));
	}

	#[test]
	fn try_find_movable_block_push_against_frame_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌  ◀▶░░
		// 2 ▌░░░░░░  ╬╬

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 1 };
		let beast_position = Coord { column: 4, row: 2 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), None);
	}

	#[test]
	fn try_find_movable_block_push_to_open_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░  ╬╬
		// 2 ▌░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 1 };
		let beast_position = Coord { column: 4, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let beast = HatchedBeast::new(beast_position);
		assert_eq!(beast.try_find_movable_block(&board, player_position), Some(Coord { column: 2, row: 1 }));
	}

	#[test]
	fn advance_squish_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌░░◀▶░░╬╬

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 1, row: 0 };
		let beast_position = Coord { column: 3, row: 0 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 0, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 0 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);
		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌░░░░╬╬

		assert_eq!(action, BeastAction::PlayerKilled);
		assert_eq!(beast.position, Coord { column: 2, row: 0 });
		assert_eq!(board[Coord { column: 3, row: 0 }], Tile::Empty);
		assert_eq!(board[Coord { column: 2, row: 0 }], Tile::HatchedBeast);
	}

	#[test]
	fn advance_a_star_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░  ╬╬
		// 2 ▌    ░░
		// 3 ▌

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 1 };
		let beast_position = Coord { column: 4, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░
		// 2 ▌    ░░╬╬
		// 3 ▌

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 3, row: 2 });
		assert_eq!(board[Coord { column: 4, row: 1 }], Tile::Empty);
		assert_eq!(board[Coord { column: 3, row: 2 }], Tile::HatchedBeast);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░
		// 2 ▌    ░░
		// 3 ▌    ╬╬

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 2, row: 3 });
		assert_eq!(board[Coord { column: 3, row: 2 }], Tile::Empty);
		assert_eq!(board[Coord { column: 2, row: 3 }], Tile::HatchedBeast);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░
		// 2 ▌  ╬╬░░
		// 3 ▌

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 1, row: 2 });
		assert_eq!(board[Coord { column: 2, row: 3 }], Tile::Empty);
		assert_eq!(board[Coord { column: 1, row: 2 }], Tile::HatchedBeast);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌╬╬  ░░
		// 2 ▌    ░░
		// 3 ▌

		assert_eq!(action, BeastAction::PlayerKilled);
		assert_eq!(beast.position, Coord { column: 0, row: 1 });
		assert_eq!(board[Coord { column: 1, row: 2 }], Tile::Empty);
		assert_eq!(board[Coord { column: 0, row: 1 }], Tile::HatchedBeast);
	}

	#[test]
	fn advance_blocked_squish_test() {
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░  ╬╬
		// 2 ▌░░░░░░

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 1 };
		let beast_position = Coord { column: 4, row: 1 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 2, row: 0 }] = Tile::Block;
		board[Coord { column: 2, row: 1 }] = Tile::Block;
		board[Coord { column: 0, row: 2 }] = Tile::Block;
		board[Coord { column: 1, row: 2 }] = Tile::Block;
		board[Coord { column: 2, row: 2 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶  ░░╬╬
		// 2 ▌░░░░░░

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 3, row: 1 });
		assert_eq!(board[Coord { column: 4, row: 1 }], Tile::Empty);
		assert_eq!(board[Coord { column: 3, row: 1 }], Tile::HatchedBeast);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌◀▶░░╬╬
		// 2 ▌░░░░░░

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 2, row: 1 });
		assert_eq!(board[Coord { column: 3, row: 1 }], Tile::Empty);
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::HatchedBeast);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌░░╬╬
		// 2 ▌░░░░░░

		assert_eq!(action, BeastAction::PlayerKilled);
		assert_eq!(beast.position, Coord { column: 1, row: 1 });
		assert_eq!(board[Coord { column: 2, row: 1 }], Tile::Empty);
		assert_eq!(board[Coord { column: 1, row: 1 }], Tile::HatchedBeast);
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

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		//   ▛▀▀▀▀▀▀▀▀▀▀▀▀▀
		// 0 ▌    ░░
		// 1 ▌  ◀▶░░╬╬
		// 2 ▌░░░░░░

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 3, row: 1 });
		assert_eq!(board[Coord { column: 3, row: 1 }], Tile::HatchedBeast);
	}

	#[test]
	fn advance_blocked_diagonal_next_test() {
		//    0 1 2 3 4 5 6 7
		// 26 ▌    ╬╬
		// 27 ▌░░░░░░
		// 28 ▌◀▶  ░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let player_position = Coord { column: 0, row: 28 };
		let beast_position = Coord { column: 2, row: 26 };

		board[player_position] = Tile::Player;
		board[beast_position] = Tile::HatchedBeast;
		board[Coord { column: 0, row: 27 }] = Tile::Block;
		board[Coord { column: 1, row: 27 }] = Tile::Block;
		board[Coord { column: 2, row: 27 }] = Tile::Block;
		board[Coord { column: 2, row: 28 }] = Tile::Block;
		board[Coord { column: 2, row: 29 }] = Tile::Block;

		let mut beast = HatchedBeast::new(beast_position);

		let action = beast.advance(&mut board, player_position);
		//    0 1 2 3 4 5 6 7
		// 26 ▌    ╬╬
		// 27 ▌░░░░░░
		// 28 ▌◀▶  ░░
		// 29 ▌    ░░
		//    ▙▄▄▄▄▄▄

		assert_eq!(action, BeastAction::Moved);
		assert_eq!(beast.position, Coord { column: 2, row: 26 });
		assert_eq!(board[Coord { column: 2, row: 26 }], Tile::HatchedBeast);
	}
}
