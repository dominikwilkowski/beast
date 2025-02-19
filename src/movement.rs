use crate::board::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
	Up,
	Right,
	Down,
	Left,
}

pub fn move_player(board: &mut Board, dir: Dir) {}

pub fn move_beasts(mut board: Board, dir: Dir) {}
pub fn move_super_beasts(mut board: Board, dir: Dir) {}
pub fn move_hatched_beasts(mut board: Board, dir: Dir) {}
