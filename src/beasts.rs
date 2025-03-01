use crate::Coord;

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

pub struct Egg {
	pub position: Coord,
}

impl Egg {
	pub fn new(position: Coord) -> Self {
		Self { position }
	}

	pub fn _advance() {}
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
