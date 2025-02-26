use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile, board::Board};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
	Up,
	Right,
	Down,
	Left,
}

fn get_next_coord(coord: Coord, dir: &Dir) -> Option<Coord> {
	let next_coord = match dir {
		Dir::Up => {
			if coord.row > 0 {
				Coord {
					row: coord.row - 1,
					column: coord.column,
				}
			} else {
				// we have arrived at the top frame
				return None;
			}
		},
		Dir::Right => {
			if coord.column < BOARD_WIDTH - 1 {
				Coord {
					row: coord.row,
					column: coord.column + 1,
				}
			} else {
				// we have arrived at the right frame
				return None;
			}
		},
		Dir::Down => {
			if coord.row < BOARD_HEIGHT - 1 {
				Coord {
					row: coord.row + 1,
					column: coord.column,
				}
			} else {
				// we have arrived at the bottom frame
				return None;
			}
		},
		Dir::Left => {
			if coord.column > 0 {
				Coord {
					row: coord.row,
					column: coord.column - 1,
				}
			} else {
				// we have arrived at the left frame
				return None;
			}
		},
	};

	Some(next_coord)
}

pub fn move_player(board: &mut Board, dir: &Dir) {
	let old_coord = board.player_position;
	if let Some(new_coord) = get_next_coord(old_coord, dir) {
		match board.data[new_coord.row][new_coord.column] {
			Tile::Empty => {
				board.data[new_coord.row][new_coord.column] = Tile::Player;
				board.data[old_coord.row][old_coord.column] = Tile::Empty;
				board.player_position = new_coord;
			},
			Tile::Block => {
				let mut next_tile = Tile::Block;
				let mut prev_coord = new_coord;

				while next_tile == Tile::Block {
					if let Some(next_coord) = get_next_coord(prev_coord, dir) {
						next_tile = board.data[next_coord.row][next_coord.column];

						match next_tile {
							Tile::Block => {
								// we need to seek deeper into the stack to find the end of this Block chain (pun not intended)
								// so nothing needs to be done here and the while loop with continue
							},
							Tile::CommonBeast | Tile::SuperBeast | Tile::Egg | Tile::EggHatching => {
								if get_next_coord(next_coord, dir).map_or(true, |coord| {
									board.data[coord.row][coord.column] == Tile::Block
										|| board.data[coord.row][coord.column] == Tile::StaticBlock
								}) {
									todo!("Squash entity")
								}
							},
							Tile::HatchedBeast => {
								if get_next_coord(next_coord, dir)
									.map_or(false, |coord| board.data[coord.row][coord.column] == Tile::StaticBlock)
								{
									todo!("Squash a hatched beast")
								}
							},
							Tile::StaticBlock | Tile::Player => {
								// Nothing happens on this move since the user is trying to push a stack of blocks against a StaticBlock | Player
							},
							Tile::Empty => {
								board.data[old_coord.row][old_coord.column] = Tile::Empty;
								board.data[new_coord.row][new_coord.column] = Tile::Player;
								board.player_position = new_coord;
								board.data[next_coord.row][next_coord.column] = Tile::Block;
							},
						}

						prev_coord = next_coord;
					} else {
						break;
					}
				}
			},
			Tile::CommonBeast | Tile::SuperBeast | Tile::HatchedBeast => {
				todo!("TODO: you ded!")
			},
			Tile::Egg | Tile::EggHatching | Tile::StaticBlock | Tile::Player => { /* nothing happens */ },
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn moving() {
		let mut board = Board {
			data: [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
			beast_locations: Vec::new(),
			super_beast_locations: Vec::new(),
			egg_locations: Vec::new(),
			player_position: Coord { row: 10, column: 5 },
		};

		// *************
		// MOVING UP
		// *************
		board.data[10][5] = Tile::Player;

		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 9, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[9][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 0, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[0][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 0, column: 5 }, "Player should not have moved");
		assert_eq!(board.data[0][5], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING RIGHT
		// *************
		board.data[0][5] = Tile::Empty;
		board.data[10][BOARD_WIDTH - 5] = Tile::Player;
		board.player_position = Coord {
			row: 10,
			column: BOARD_WIDTH - 5,
		};

		move_player(&mut board, &Dir::Right);
		assert_eq!(
			board.player_position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 4
			},
			"Player should move right one column"
		);
		assert_eq!(board.data[10][BOARD_WIDTH - 4], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		assert_eq!(
			board.player_position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 1
			},
			"Player should move right one column"
		);
		assert_eq!(board.data[10][BOARD_WIDTH - 1], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		assert_eq!(
			board.player_position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 1
			},
			"Player should not have moved"
		);
		assert_eq!(board.data[10][BOARD_WIDTH - 1], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING DOWN
		// *************
		board.data[10][BOARD_WIDTH - 1] = Tile::Empty;
		board.data[BOARD_HEIGHT - 3][5] = Tile::Player;
		board.player_position = Coord {
			row: BOARD_HEIGHT - 3,
			column: 5,
		};

		move_player(&mut board, &Dir::Down);
		assert_eq!(
			board.player_position,
			Coord {
				row: BOARD_HEIGHT - 2,
				column: 5
			},
			"Player should move down one row"
		);
		assert_eq!(board.data[BOARD_HEIGHT - 2][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Down);
		assert_eq!(
			board.player_position,
			Coord {
				row: BOARD_HEIGHT - 1,
				column: 5
			},
			"Player should move down one row"
		);
		assert_eq!(board.data[BOARD_HEIGHT - 1][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		assert_eq!(
			board.player_position,
			Coord {
				row: BOARD_HEIGHT - 1,
				column: 5
			},
			"Player should not have moved"
		);
		assert_eq!(board.data[BOARD_HEIGHT - 1][5], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING LEFT
		// *************
		board.data[BOARD_HEIGHT - 1][5] = Tile::Empty;
		board.data[10][5] = Tile::Player;
		board.player_position = Coord { row: 10, column: 5 };
		board.data[10][5] = Tile::Player;

		move_player(&mut board, &Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 4 }, "Player should move left one column");
		assert_eq!(board.data[10][4], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 0 }, "Player should move left one column");
		assert_eq!(board.data[10][0], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, &Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 0 }, "Player should not have moved");
		assert_eq!(board.data[10][0], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
	}

	#[test]
	fn push_block() {
		let mut board = Board {
			data: [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
			beast_locations: Vec::new(),
			super_beast_locations: Vec::new(),
			egg_locations: Vec::new(),
			player_position: Coord { row: 5, column: 5 },
		};
		board.data[5][5] = Tile::Player;
		board.data[3][5] = Tile::Block;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ░░
		// 4 ▌
		// 5 ▌        ◄►

		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 4, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[4][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::Block, "The Block hasn't moved");

		// 1 ▌
		// 2 ▌
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌

		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 3, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[3][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[2][5], Tile::Block, "The Block has moved up one row");

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ◄►
		// 4 ▌
		// 5 ▌

		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Left);
		assert_eq!(board.player_position, Coord { row: 2, column: 5 }, "Player should moved right, up and left");
		assert_eq!(board.data[2][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[2][4], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌      ░░◄►
		// 3 ▌
		// 4 ▌
		// 5 ▌

		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Down);
		assert_eq!(board.player_position, Coord { row: 2, column: 4 }, "Player should moved up, left and down");
		assert_eq!(board.data[2][4], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][4], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌      ◄►
		// 3 ▌      ░░
		// 4 ▌
		// 5 ▌

		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Right);
		assert_eq!(board.player_position, Coord { row: 3, column: 4 }, "Player should moved left, down and right");
		assert_eq!(board.data[3][4], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌
		// 3 ▌      ◄►░░
		// 4 ▌
		// 5 ▌
	}

	#[test]
	fn push_block_chain() {
		let mut board = Board {
			data: [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
			beast_locations: Vec::new(),
			super_beast_locations: Vec::new(),
			egg_locations: Vec::new(),
			player_position: Coord { row: 10, column: 0 },
		};
		board.data[10][0] = Tile::Player;
		board.data[9][0] = Tile::Block;
		board.data[8][0] = Tile::Block;
		board.data[7][0] = Tile::Block;
		board.data[6][0] = Tile::Block;
		board.data[5][0] = Tile::Empty;
		board.data[4][0] = Tile::StaticBlock;

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌░░
		// 10 ▌◄►

		// move up
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 9, column: 0 }, "Player should move up one row");
		assert_eq!(board.data[10][0], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board.data[9][0], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board.data[8][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[7][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[6][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[5][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[4][0], Tile::StaticBlock, "The StaticBlock hasn't moved");

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌░░
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌◄►
		// 10 ▌

		// move up again
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 9, column: 0 }, "Player should not move");
		assert_eq!(board.data[9][0], Tile::Player, "Player tile should not move");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board.data[8][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[7][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[6][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[5][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[4][0], Tile::StaticBlock, "The StaticBlock should not move");

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌░░
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌◄►
		// 10 ▌

		// now let's cheat and remove the static block
		board.data[4][0] = Tile::Empty;
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 4, column: 0 }, "Player should move up four rows");
		assert_eq!(board.data[9][0], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board.data[4][0], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board.data[3][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[2][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[1][0], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board.data[0][0], Tile::Block, "The Blocks should have moved up");

		//    ▛▀
		//  0 ▌░░
		//  1 ▌░░
		//  2 ▌░░
		//  3 ▌░░
		//  4 ▌◄►
		//  5 ▌
		//  6 ▌
		//  7 ▌
		//  8 ▌
		//  9 ▌
		// 10 ▌

		// now that we're up against the wall let's move up one more time
		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 4, column: 0 }, "Player should not move");
		assert_eq!(board.data[4][0], Tile::Player, "Player tile should not move");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board.data[3][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[2][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[1][0], Tile::Block, "The Blocks should not move");
		assert_eq!(board.data[0][0], Tile::Block, "The Blocks should not move");

		//    ▛▀
		//  0 ▌░░
		//  1 ▌░░
		//  2 ▌░░
		//  3 ▌░░
		//  4 ▌◄►
		//  5 ▌
		//  6 ▌
		//  7 ▌
		//  8 ▌
		//  9 ▌
		// 10 ▌
	}

	#[test]
	fn push_static_block() {
		let mut board = Board {
			data: [[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
			beast_locations: Vec::new(),
			super_beast_locations: Vec::new(),
			egg_locations: Vec::new(),
			player_position: Coord { row: 5, column: 5 },
		};
		board.data[5][5] = Tile::Player;
		board.data[3][5] = Tile::StaticBlock;

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌
		// 5 ▌        ◄►

		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 4, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[5][5], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board.data[4][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌        ◄►
		// 5 ▌

		move_player(&mut board, &Dir::Up);
		assert_eq!(board.player_position, Coord { row: 4, column: 5 }, "Player should not have moved");
		assert_eq!(board.data[4][5], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌        ◄►
		// 5 ▌

		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		assert_eq!(board.player_position, Coord { row: 3, column: 6 }, "Player should now be next to the StaticBlock");
		assert_eq!(board.data[3][6], Tile::Player, "Player tile should have moved to the right and up");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓◄►
		// 4 ▌
		// 5 ▌

		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Up);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Down);
		assert_eq!(board.player_position, Coord { row: 2, column: 5 }, "Player should now be above the StaticBlock");
		assert_eq!(board.data[2][5], Tile::Player, "Player tile should have moved up and left");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌        ◄►
		// 3 ▌        ▓▓
		// 4 ▌
		// 5 ▌

		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Left);
		move_player(&mut board, &Dir::Down);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		move_player(&mut board, &Dir::Right);
		assert_eq!(board.player_position, Coord { row: 3, column: 4 }, "Player should now be above the StaticBlock");
		assert_eq!(board.data[3][4], Tile::Player, "Player tile should have moved up and left");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board.data[3][5], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌      ◄►▓▓
		// 4 ▌
		// 5 ▌
	}
}
