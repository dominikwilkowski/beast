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
		},
		Tile::Block => {
			todo!("TODO: seek through till end till we either find Empty or StaticBlock")
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
