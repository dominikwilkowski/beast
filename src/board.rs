pub struct Coord {
	x: usize,
	y: usize,
}

pub struct Partial<'a> {
	coord: Coord,
	replacement: &'a str,
}

pub struct Board {}

impl Board {
	pub fn new() -> Self {
		Self {}
	}
	fn terrain_gen() {}

	pub fn render_full() {}
	pub fn render_partial(partials: Vec<Partial>) {}
}
