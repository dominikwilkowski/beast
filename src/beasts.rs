use std::time::Instant;

use crate::{Coord, levels::LevelConfig};

// TODO: add trait for beast
// score:
// egg: 2
// CommonBeast: 2
// SuperBeast: 6
// HatchedBeast: 2
// win level: 7

pub struct CommonBeast {
	pub position: Coord,
}

impl CommonBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn _advance() {}
}

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn _advance() {}
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
}

pub struct HatchedBeast {
	pub position: Coord,
}

impl HatchedBeast {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn _advance() {}
}
