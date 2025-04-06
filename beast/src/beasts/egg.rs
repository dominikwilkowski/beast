use beast_common::levels::LevelConfig;
use std::time::Instant;

use crate::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HatchingState {
	Incubating,
	Hatching(Coord, Instant),
	Hatched(Coord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Egg {
	pub position: Coord,
	pub instant: Instant,
	state: HatchingState,
}

impl Egg {
	pub fn new(position: Coord, instant: Instant) -> Self {
		Self {
			position,
			instant,
			state: HatchingState::Incubating,
		}
	}

	pub fn hatch(&mut self, level: LevelConfig) -> HatchingState {
		if self.instant.elapsed() >= level.egg_hatching_time {
			HatchingState::Hatched(self.position)
		} else if self.instant.elapsed() >= (level.egg_hatching_time / 10) * 8
			&& self.state != HatchingState::Hatching(self.position, self.instant)
		{
			self.state = HatchingState::Hatching(self.position, self.instant);
			HatchingState::Hatching(self.position, self.instant)
		} else {
			HatchingState::Incubating
		}
	}

	pub fn get_score() -> u16 {
		2
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::Duration;

	#[test]
	fn test_egg_creation() {
		let position = Coord { column: 5, row: 10 };
		let now = Instant::now();
		let egg = Egg::new(position, now);

		assert_eq!(egg.position, position, "The new instance has the right position");
		assert_eq!(egg.instant, now, "The new instance has the right time");
		assert_eq!(egg.state, HatchingState::Incubating, "The new instance has the right state");
	}

	#[test]
	fn test_egg_get_score() {
		assert_eq!(Egg::get_score(), 2, "The egg's score is correct");
	}

	#[test]
	fn test_egg_hatch_incubating() {
		let position = Coord { column: 5, row: 10 };
		let now = Instant::now();
		let mut egg = Egg::new(position, now);

		let level = LevelConfig {
			blocks: 10,
			static_blocks: 5,
			common_beasts: 3,
			super_beasts: 1,
			eggs: 4,
			egg_hatching_time: Duration::from_secs(100),
			beast_starting_distance: 5,
			time: Duration::from_secs(300),
			completion_score: 100,
		};

		assert_eq!(egg.hatch(level), HatchingState::Incubating, "The egg is still incubating");
	}

	#[test]
	fn test_egg_hatch_hatching() {
		let position = Coord { column: 5, row: 10 };
		let past_time = Instant::now() - Duration::from_secs(80);
		let mut egg = Egg::new(position, past_time);

		let level = LevelConfig {
			blocks: 10,
			static_blocks: 5,
			common_beasts: 3,
			super_beasts: 1,
			eggs: 4,
			egg_hatching_time: Duration::from_secs(100),
			beast_starting_distance: 5,
			time: Duration::from_secs(300),
			completion_score: 100,
		};

		assert_eq!(
			egg.hatch(level.clone()),
			HatchingState::Hatching(position, past_time),
			"The egg should be hatching after 80% of the time has passed"
		);
		assert_eq!(
			egg.hatch(level.clone()),
			HatchingState::Incubating,
			"All next calls to hatch should return Incubating 1"
		);
		assert_eq!(
			egg.hatch(level.clone()),
			HatchingState::Incubating,
			"All next calls to hatch should return Incubating 2"
		);
		assert_eq!(egg.hatch(level), HatchingState::Incubating, "All next calls to hatch should return Incubating 3");
	}

	#[test]
	fn test_egg_hatch_hatched() {
		let position = Coord { column: 5, row: 10 };
		let past_time = Instant::now() - Duration::from_secs(110);
		let mut egg = Egg::new(position, past_time);

		let level = LevelConfig {
			blocks: 10,
			static_blocks: 5,
			common_beasts: 3,
			super_beasts: 1,
			eggs: 4,
			egg_hatching_time: Duration::from_secs(100),
			beast_starting_distance: 5,
			time: Duration::from_secs(300),
			completion_score: 100,
		};

		assert_eq!(
			egg.hatch(level.clone()),
			HatchingState::Hatched(position),
			"The egg should have hatched after 110% of the time has passed"
		);
	}
}
