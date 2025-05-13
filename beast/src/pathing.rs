//! pathfinding utilities for the game reused by at least two modules

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Dir, Tile, board::Board};

/// this method returns the end coordinate of a chain of blocks which will be anything but Tile::Block
pub fn get_end_of_block_chain(board: &Board, start: &Coord, dir: &Dir) -> Option<(Coord, u64)> {
	let mut next_tile = Tile::Block;
	let mut end_coord = *start;
	let mut blocks_moved = 0;

	while next_tile == Tile::Block {
		if let Some(next_coord) = get_next_coord(&end_coord, dir) {
			next_tile = board[next_coord];
			end_coord = next_coord;

			match next_tile {
				Tile::Block => {
					blocks_moved += 1;
					// we need to seek deeper into the stack to find the end of this Block chain (pun not intended)
					// so nothing needs to be done here and the while loop with continue
				},
				_ => {
					break;
				},
			}
		} else {
			// we hit the frame
			return None;
		}
	}

	Some((end_coord, blocks_moved))
}

/// this method returns the next coordinate in the direction specified respecting the board boundaries
pub fn get_next_coord(coord: &Coord, dir: &Dir) -> Option<Coord> {
	match dir {
		Dir::Up if coord.row > 0 => Some(Coord {
			row: coord.row - 1,
			column: coord.column,
		}),
		Dir::Right if coord.column < BOARD_WIDTH - 1 => Some(Coord {
			row: coord.row,
			column: coord.column + 1,
		}),
		Dir::Down if coord.row < BOARD_HEIGHT - 1 => Some(Coord {
			row: coord.row + 1,
			column: coord.column,
		}),
		Dir::Left if coord.column > 0 => Some(Coord {
			row: coord.row,
			column: coord.column - 1,
		}),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_end_of_block_chain_up_test() {
		// ├┤
		// ░░
		// ░░
		// ◀▶

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let start = Coord { column: 0, row: 3 };
		let dir = Dir::Up;

		board.buffer[0][0] = Tile::CommonBeast;
		board.buffer[1][0] = Tile::Block;
		board.buffer[2][0] = Tile::Block;

		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 0 }, 2)),
			"Should seek to the position where the CommonBeast is"
		);

		board.buffer[3][0] = Tile::SuperBeast;
		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 0 }, 2)),
			"Should seek to the position where the SuperBeast is"
		);

		// ▀▀
		// ░░
		// ░░
		// ░░
		// ◀▶
		board.buffer[0][0] = Tile::Block;

		assert_eq!(get_end_of_block_chain(&board, &start, &dir), None, "When hitting the frame return None");
	}

	#[test]
	fn get_end_of_block_chain_right_test() {
		// ◀▶░░░░░░├┤

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let start = Coord { column: 0, row: 0 };
		let dir = Dir::Right;

		board.buffer[0][1] = Tile::Block;
		board.buffer[0][2] = Tile::Block;
		board.buffer[0][3] = Tile::Block;
		board.buffer[0][4] = Tile::CommonBeast;

		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 4, row: 0 }, 3)),
			"Should seek to the position where the CommonBeast is"
		);

		board.buffer[0][4] = Tile::StaticBlock;
		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 4, row: 0 }, 3)),
			"Should seek to the position where the StaticBlock is"
		);

		// ◀▶░░░░░░▐
		let start = Coord {
			column: BOARD_WIDTH - 4,
			row: 0,
		};

		board.buffer[0][BOARD_WIDTH - 3] = Tile::Block;
		board.buffer[0][BOARD_WIDTH - 2] = Tile::Block;
		board.buffer[0][BOARD_WIDTH - 1] = Tile::Block;

		assert_eq!(get_end_of_block_chain(&board, &start, &dir), None, "When hitting the frame return None");
	}

	#[test]
	fn get_end_of_block_chain_down_test() {
		// ◀▶
		// ░░
		// ░░
		// ├┤

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let start = Coord { column: 0, row: 0 };
		let dir = Dir::Down;

		board.buffer[1][0] = Tile::Block;
		board.buffer[2][0] = Tile::Block;
		board.buffer[3][0] = Tile::CommonBeast;

		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 3 }, 2)),
			"Should seek to the position where the CommonBeast is"
		);

		board.buffer[3][0] = Tile::HatchedBeast;
		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 3 }, 2)),
			"Should seek to the position where the HatchedBeast is"
		);

		// ◀▶
		// ░░
		// ░░
		// ▄▄
		let start = Coord {
			column: 0,
			row: BOARD_HEIGHT - 3,
		};

		board.buffer[BOARD_HEIGHT - 2][0] = Tile::Block;
		board.buffer[BOARD_HEIGHT - 1][0] = Tile::Block;

		assert_eq!(get_end_of_block_chain(&board, &start, &dir), None, "When hitting the frame return None");
	}

	#[test]
	fn get_end_of_block_chain_left_test() {
		// ├┤░░░░░░◀▶

		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let start = Coord { column: 6, row: 0 };
		let dir = Dir::Left;

		board.buffer[0][0] = Tile::CommonBeast;
		board.buffer[0][1] = Tile::Block;
		board.buffer[0][2] = Tile::Block;
		board.buffer[0][3] = Tile::Block;
		board.buffer[0][4] = Tile::Block;
		board.buffer[0][5] = Tile::Block;

		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 0 }, 5)),
			"Should seek to the position where the CommonBeast is"
		);

		board.buffer[0][0] = Tile::Empty;
		assert_eq!(
			get_end_of_block_chain(&board, &start, &dir),
			Some((Coord { column: 0, row: 0 }, 5)),
			"Should seek to the position where the Empty is"
		);

		// ▌░░░░░░░░◀▶
		board.buffer[0][0] = Tile::Block;

		assert_eq!(get_end_of_block_chain(&board, &start, &dir), None, "When hitting the frame return None");
	}

	#[test]
	fn get_next_coord_test() {
		assert_eq!(
			get_next_coord(&Coord { row: 5, column: 5 }, &Dir::Up),
			Some(Coord { row: 4, column: 5 }),
			"The next tile is above"
		);
		assert_eq!(
			get_next_coord(&Coord { row: 5, column: 5 }, &Dir::Right),
			Some(Coord { row: 5, column: 6 }),
			"The next tile is right"
		);
		assert_eq!(
			get_next_coord(&Coord { row: 5, column: 5 }, &Dir::Down),
			Some(Coord { row: 6, column: 5 }),
			"The next tile is below"
		);
		assert_eq!(
			get_next_coord(&Coord { row: 5, column: 5 }, &Dir::Left),
			Some(Coord { row: 5, column: 4 }),
			"The next tile is left"
		);
	}

	#[test]
	fn get_next_coord_edge_test() {
		let coord = Coord { row: 0, column: 5 };
		assert_eq!(get_next_coord(&coord, &Dir::Up), None, "The next tile is out of bounds");

		let coord = Coord {
			row: 0,
			column: BOARD_WIDTH - 1,
		};
		assert_eq!(get_next_coord(&coord, &Dir::Right), None, "The next tile is out of bounds");

		let coord = Coord {
			row: BOARD_HEIGHT - 1,
			column: 5,
		};
		assert_eq!(get_next_coord(&coord, &Dir::Down), None, "The next tile is out of bounds");

		let coord = Coord { row: 5, column: 0 };
		assert_eq!(get_next_coord(&coord, &Dir::Left), None, "The next tile is out of bounds");
	}
}
