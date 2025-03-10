use std::time::Instant;

use crate::{Coord, levels::LevelConfig};

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
