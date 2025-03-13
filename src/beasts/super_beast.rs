use std::{cmp::Ordering, collections::HashMap};

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile,
	beasts::{Beast, BeastAction},
	board::Board,
};

pub struct SuperBeast {
	pub position: Coord,
}

impl SuperBeast {
	/// return the Chebyshev distance on a 2D board
	fn heuristic(a: Coord, b: Coord) -> i32 {
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

	/// return if a tile is walkable
	fn is_walkable(tile: Tile) -> bool {
		matches!(tile, Tile::Empty | Tile::Player)
	}

	/// returns all walkable neighbors (8-directional) for a given coordinate.
	fn neighbors(board: &Board, position: &Coord, player_position: &Coord) -> Vec<Coord> {
		let mut result = Vec::with_capacity(8);

		// top row
		let left_top: Coord = Coord {
			column: position.column.saturating_sub(1),
			row: position.row.saturating_sub(1),
		};
		let middle_top: Coord = Coord {
			column: position.column,
			row: position.row.saturating_sub(1),
		};
		let right_top: Coord = Coord {
			column: std::cmp::min(position.column + 1, BOARD_WIDTH - 1),
			row: position.row.saturating_sub(1),
		};

		// middle row
		let left_middle: Coord = Coord {
			column: position.column.saturating_sub(1),
			row: position.row,
		};
		let right_middle: Coord = Coord {
			column: std::cmp::min(position.column + 1, BOARD_WIDTH - 1),
			row: position.row,
		};

		// bottom row
		let left_bottom: Coord = Coord {
			column: position.column.saturating_sub(1),
			row: std::cmp::min(position.row + 1, BOARD_HEIGHT - 1),
		};
		let middle_bottom: Coord = Coord {
			column: position.column,
			row: std::cmp::min(position.row + 1, BOARD_HEIGHT - 1),
		};
		let right_bottom: Coord = Coord {
			column: std::cmp::min(position.column + 1, BOARD_WIDTH - 1),
			row: std::cmp::min(position.row + 1, BOARD_HEIGHT - 1),
		};

		match (player_position.column.cmp(&position.column), player_position.row.cmp(&position.row)) {
			(Ordering::Equal, Ordering::Greater) => {
				// player is straight below
				// 6 8  7
				// 4 ├┤ 5
				// 2 ◀▶ 3
				if Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}

				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
			},
			(Ordering::Equal, Ordering::Less) => {
				// player is straight above
				// 2 ◀▶ 3
				// 4 ├┤ 5
				// 6 8  7
				if Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}

				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
			},
			(Ordering::Less, Ordering::Equal) => {
				// player is straight left
				// 2 4  6
				// ◀▶├┤ 8
				// 3 5  7
				if Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}

				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
			},
			(Ordering::Greater, Ordering::Equal) => {
				// player is straight right
				// 6 4  2
				// 8 ├┤◀▶
				// 7 5  3
				if Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}

				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
			},
			(Ordering::Greater, Ordering::Greater) => {
				// player is below right
				// 8 7  5
				// 6 ├┤ 3
				// 4 2 ◀▶
				if Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}

				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
			},
			(Ordering::Greater, Ordering::Less) => {
				// player is above right
				// 4 2 ◀▶
				// 6 ├┤ 3
				// 8 7  5
				if Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}

				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
			},
			(Ordering::Less, Ordering::Greater) => {
				// player is below left
				// 4 6  8
				// 2 ├┤ 7
				// ◀▶ 3 5
				if Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}

				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&left_top) && Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
			},
			(Ordering::Less, Ordering::Less) => {
				// player is above left
				// ◀▶ 3 5
				// 2 ├┤ 7
				// 4 6  8
				if Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}

				if !result.contains(&left_middle) && Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if !result.contains(&right_bottom) && Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
			},
			(Ordering::Equal, Ordering::Equal) => {
				// Player is at the same position.
				if Self::is_walkable(board[left_top]) {
					result.push(left_top);
				}

				if Self::is_walkable(board[middle_top]) {
					result.push(middle_top);
				}
				if Self::is_walkable(board[right_top]) {
					result.push(right_top);
				}
				if Self::is_walkable(board[left_middle]) {
					result.push(left_middle);
				}
				if Self::is_walkable(board[right_middle]) {
					result.push(right_middle);
				}
				if Self::is_walkable(board[left_bottom]) {
					result.push(left_bottom);
				}
				if Self::is_walkable(board[middle_bottom]) {
					result.push(middle_bottom);
				}
				if Self::is_walkable(board[right_bottom]) {
					result.push(right_bottom);
				}
			},
		};

		result
	}

	/// an A* pathfinding implementation
	fn astar(&self, board: &Board, start: Coord, goal: Coord) -> Option<Vec<Coord>> {
		let mut open_set = vec![start];

		let mut came_from: HashMap<Coord, Coord> = HashMap::new();

		let mut g_score: HashMap<Coord, i32> = HashMap::new();
		g_score.insert(start, 0);

		let mut f_score: HashMap<Coord, i32> = HashMap::new();
		f_score.insert(start, Self::heuristic(start, goal));

		while !open_set.is_empty() {
			let current = *open_set.iter().min_by_key(|coord| f_score.get(coord).unwrap_or(&i32::MAX)).unwrap();

			if current == goal {
				return Some(Self::reconstruct_path(&came_from, current));
			}

			open_set.retain(|&c| c != current);

			for neighbor in Self::neighbors(board, &current, &goal) {
				let tentative_g_score = g_score.get(&current).unwrap_or(&i32::MAX) + 1;
				if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
					came_from.insert(neighbor, current);
					g_score.insert(neighbor, tentative_g_score);
					f_score.insert(neighbor, tentative_g_score + Self::heuristic(neighbor, goal));
					if !open_set.contains(&neighbor) {
						open_set.push(neighbor);
					}
				}
			}
		}

		// TODO: walk towards the player when no path was found
		None
	}
}

impl Beast for SuperBeast {
	fn new(position: Coord) -> Self {
		Self { position }
	}

	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction {
		if let Some(path) = self.astar(board, self.position, player_position) {
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
