use std::time::Instant;

use crate::{Coord, board::Board, levels::LevelConfig};

// TODO: add trait for beast

pub enum BeastAction {
	PlayerKilled,
	Movement,
}

pub struct CommonBeast {
	pub position: Coord,
}

impl CommonBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn advance(&mut self, _board: &mut Board) -> BeastAction {
		BeastAction::Movement
	}

	pub fn get_score() -> u16 {
		2
	}
}

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn advance(&mut self, _board: &mut Board) -> BeastAction {
		BeastAction::Movement
	}

	pub fn get_score() -> u16 {
		6
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HatchingState {
	Incubating,
	Hatching(Coord, Instant),
	Hatched(Coord),
}

pub struct Egg {
	pub position: Coord,
	pub instant: Instant,
}

impl Egg {
	pub fn new(position: Coord, instant: Instant) -> Self {
		Self { position, instant }
	}

	pub fn _advance() {}

	pub fn hatch(&self, level: LevelConfig) -> HatchingState {
		if self.instant.elapsed() >= level.egg_hatching_time {
			HatchingState::Hatched(self.position)
		} else if self.instant.elapsed() >= (level.egg_hatching_time / 10) * 8 {
			HatchingState::Hatching(self.position, self.instant)
		} else {
			HatchingState::Incubating
		}
	}

	pub fn get_score() -> u16 {
		2
	}
}

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn advance(&mut self, _board: &mut Board) -> BeastAction {
		BeastAction::Movement
	}

	pub fn get_score() -> u16 {
		2
	}
}
