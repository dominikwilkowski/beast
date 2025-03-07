use crate::Coord;

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
