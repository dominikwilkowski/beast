use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile, board::Board};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
	Up,
	Right,
	Down,
	Left,
}

pub fn move_player(board: &mut Board, dir: Dir) {
	let old_coord = board.player_position;
	let new_coord = match dir {
		Dir::Up => Coord {
			row: old_coord.row.saturating_sub(1),
			column: old_coord.column,
		},
		Dir::Right => Coord {
			row: old_coord.row,
			column: old_coord.column.saturating_add(1).min(BOARD_WIDTH - 1),
		},
		Dir::Down => Coord {
			row: old_coord.row.saturating_add(1).min(BOARD_HEIGHT - 1),
			column: old_coord.column,
		},
		Dir::Left => Coord {
			row: old_coord.row,
			column: old_coord.column.saturating_sub(1),
		},
	};

	match board.data[new_coord.row][new_coord.column] {
		Tile::Empty => {
			board.data[new_coord.row][new_coord.column] = Tile::Player;
			board.data[old_coord.row][old_coord.column] = Tile::Empty;
			board.player_position = new_coord;
		},
		Tile::Block => {
			let mut next_tile = Tile::Block;
			let mut next_coord = new_coord;

			while next_tile == Tile::Block {
				next_coord = match dir {
					Dir::Up => {
						if next_coord.row > 0 {
							Coord {
								row: next_coord.row - 1,
								column: next_coord.column,
							}
						} else {
							// we have arrived at the top frame
							break;
						}
					},
					Dir::Right => {
						if next_coord.column < BOARD_WIDTH - 1 {
							Coord {
								row: next_coord.row,
								column: next_coord.column + 1,
							}
						} else {
							// we have arrived at the right frame
							break;
						}
					},
					Dir::Down => {
						if next_coord.row < BOARD_HEIGHT - 1 {
							Coord {
								row: next_coord.row + 1,
								column: next_coord.column,
							}
						} else {
							// we have arrived at the bottom frame
							break;
						}
					},
					Dir::Left => {
						if next_coord.column > 0 {
							Coord {
								row: next_coord.row,
								column: next_coord.column - 1,
							}
						} else {
							// we have arrived at the left frame
							break;
						}
					},
				};

				next_tile = board.data[next_coord.row][next_coord.column];

				match next_tile {
					Tile::Block => {
						// we need to seek deeper into the stack to find the end of this Block chain (pun not intended)
						// so nothing needs to be done here and the while loop with continue
					},
					Tile::CommonBeast | Tile::SuperBeast | Tile::Egg | Tile::EggHatching => {
						todo!("Squash a beast/egg if block after is Block | StaticBlock")
					},
					Tile::HatchedBeast => {
						todo!("Squash a hatched beast if next block is static")
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
			}
		},
		Tile::CommonBeast | Tile::SuperBeast | Tile::HatchedBeast => {
			todo!("TODO: you ded!")
		},
		Tile::Egg | Tile::EggHatching | Tile::StaticBlock | Tile::Player => { /* nothing happens */ },
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

		move_player(&mut board, Dir::Up);
		assert_eq!(board.player_position, Coord { row: 9, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[9][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		assert_eq!(board.player_position, Coord { row: 0, column: 5 }, "Player should move up one row");
		assert_eq!(board.data[0][5], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
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

		move_player(&mut board, Dir::Right);
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

		move_player(&mut board, Dir::Right);
		move_player(&mut board, Dir::Right);
		move_player(&mut board, Dir::Right);
		move_player(&mut board, Dir::Right);
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
		move_player(&mut board, Dir::Right);
		move_player(&mut board, Dir::Right);
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

		move_player(&mut board, Dir::Down);
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

		move_player(&mut board, Dir::Down);
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

		move_player(&mut board, Dir::Down);
		move_player(&mut board, Dir::Down);
		move_player(&mut board, Dir::Down);
		move_player(&mut board, Dir::Down);
		move_player(&mut board, Dir::Down);
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

		move_player(&mut board, Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 4 }, "Player should move left one column");
		assert_eq!(board.data[10][4], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, Dir::Left);
		move_player(&mut board, Dir::Left);
		move_player(&mut board, Dir::Left);
		move_player(&mut board, Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 0 }, "Player should move left one column");
		assert_eq!(board.data[10][0], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		move_player(&mut board, Dir::Left);
		assert_eq!(board.player_position, Coord { row: 10, column: 0 }, "Player should not have moved");
		assert_eq!(board.data[10][0], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
	}

	#[test]
	fn push_up() {
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
		move_player(&mut board, Dir::Up);
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
		move_player(&mut board, Dir::Up);
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
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
		move_player(&mut board, Dir::Up);
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
		move_player(&mut board, Dir::Up);
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
}
