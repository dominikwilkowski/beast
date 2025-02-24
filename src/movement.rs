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
							return;
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
							return;
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
							return;
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
							return;
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
						todo!("Gotta now execute the push")
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

// pub fn _move_beasts(mut _board: Board, _dir: Dir) {}
// pub fn _move_super_beasts(mut _board: Board, _dir: Dir) {}
// pub fn _move_hatched_beasts(mut _board: Board, _dir: Dir) {}
