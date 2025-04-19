//! pathfinding utilities for the game reused by at least two modules

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

#[cfg(test)]
mod tests {
	// use super::*;
	// TODO
}
