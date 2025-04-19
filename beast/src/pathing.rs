use std::cmp::Ordering;

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Dir, Tile, board::Board};

pub fn get_end_of_block_chain(board: &Board, start: &Coord, dir: &Dir) -> Option<(Coord, u64)> {
	let mut next_tile = Tile::Block;
	let mut end_coord = *start;
	let mut blocks_moved = 1;

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

pub fn get_dir(from_position: Coord, to_position: Coord) -> Dir {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_dir_test() {
		assert_eq!(get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 6 }), Dir::Down);
		assert_eq!(get_dir(Coord { column: 5, row: 5 }, Coord { column: 6, row: 5 }), Dir::Right);
		assert_eq!(get_dir(Coord { column: 5, row: 5 }, Coord { column: 5, row: 4 }), Dir::Up);
		assert_eq!(get_dir(Coord { column: 5, row: 5 }, Coord { column: 4, row: 5 }), Dir::Left);
	}
}
