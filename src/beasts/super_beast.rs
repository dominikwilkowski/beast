use std::collections::HashMap;

use crate::{
	Coord, Tile,
	beasts::{Beast, BeastAction, get_walkable_coords},
	board::Board,
};

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	/// return the Chebyshev distance on a 2D board
	fn heuristic(a: &Coord, b: &Coord) -> i32 {
		let distance_column = (a.column as i32 - b.column as i32).abs();
		let distance_row = (a.row as i32 - b.row as i32).abs();

		distance_column.max(distance_row)
	}

	/// reconstructs the path from start to goal using the came_from map
	fn reconstruct_path(came_from: &HashMap<Coord, Coord>, mut current: Coord) -> Vec<Coord> {
		let mut reconstructed_path = vec![current];
		while let Some(&prev) = came_from.get(&current) {
			current = prev;
			reconstructed_path.push(current);
		}
		reconstructed_path.reverse();

		reconstructed_path
	}

	/// an A* pathfinding implementation
	fn astar(&self, board: &Board, start: Coord, goal: &Coord) -> Option<Vec<Coord>> {
		let mut open_set = vec![start];

		let mut came_from: HashMap<Coord, Coord> = HashMap::new();

		let mut g_score: HashMap<Coord, i32> = HashMap::new();
		g_score.insert(start, 0);

		let mut f_score: HashMap<Coord, i32> = HashMap::new();
		f_score.insert(start, Self::heuristic(&start, goal));

		while !open_set.is_empty() {
			let current = *open_set.iter().min_by_key(|coord| f_score.get(coord).unwrap_or(&i32::MAX)).unwrap();

			if current == *goal {
				return Some(Self::reconstruct_path(&came_from, current));
			}

			open_set.retain(|&c| c != current);

			for neighbor in get_walkable_coords(board, &current, goal, true) {
				let tentative_g_score = g_score.get(&current).unwrap_or(&i32::MAX) + 1;
				if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
					came_from.insert(neighbor, current);
					g_score.insert(neighbor, tentative_g_score);
					f_score.insert(neighbor, tentative_g_score + Self::heuristic(&neighbor, goal));
					if !open_set.contains(&neighbor) {
						open_set.push(neighbor);
					}
				}
			}
		}

		// when there is no path we at least still go towards the player
		for neighbor in get_walkable_coords(board, &start, goal, true) {
			match board[neighbor] {
				Tile::Empty | Tile::Player => {
					return Some(vec![Coord { column: 0, row: 0 }, neighbor]);
				},
				Tile::Block
				| Tile::StaticBlock
				| Tile::CommonBeast
				| Tile::SuperBeast
				| Tile::HatchedBeast
				| Tile::Egg(_)
				| Tile::EggHatching(_) => {},
			}
		}

		None
	}
}

impl Beast for SuperBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		if let Some(path) = self.astar(board, self.position, &player_position) {
			if path.len() > 1 {
				// the first item is our own position
				let next_step = path[1];

				match board[next_step] {
					Tile::Player => {
						board[next_step] = Tile::SuperBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::PlayerKilled;
					},
					Tile::Empty => {
						board[next_step] = Tile::SuperBeast;
						board[self.position] = Tile::Empty;
						self.position = next_step;
						return BeastAction::Moved;
					},
					_ => {},
				}
			}
		}

		BeastAction::Moved
	}

	fn get_score() -> u16 {
		6
	}
}
