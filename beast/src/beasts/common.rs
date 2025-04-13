use std::{cmp::Ordering, collections::HashMap};

use crate::{BOARD_HEIGHT, BOARD_WIDTH, Coord, Tile, board::Board};

/// the action a beast can take
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeastAction {
	/// the beast has killed the player
	PlayerKilled,
	/// the beast has moved to a new position
	Moved,
}

/// this trait defines the common behavior of all beasts in the game
pub trait Beast {
	/// creates a new instance of the beast and stores its position
	fn new(position: Coord) -> Self;

	/// advances the beast's position and returns the action taken
	fn advance(&mut self, board: &mut Board, player_position: Coord) -> BeastAction;

	/// returns the score for when this beast is crushed
	fn get_score() -> u16;

	/// return if a tile is walkable
	fn is_walkable_tile(tile: &Tile) -> bool {
		matches!(tile, Tile::Empty | Tile::Player)
	}

	/// returns all walkable neighbors (8-directional) for a given position
	fn get_walkable_coords(board: &Board, position: &Coord, player_position: &Coord, check_tiles: bool) -> Vec<Coord> {
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
				if Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}

				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
			},
			(Ordering::Equal, Ordering::Less) => {
				// player is straight above
				// 2 ◀▶ 3
				// 4 ├┤ 5
				// 6 8  7
				if Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}

				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
			},
			(Ordering::Less, Ordering::Equal) => {
				// player is straight left
				// 2 4  6
				// ◀▶├┤ 8
				// 3 5  7
				if Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}

				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
			},
			(Ordering::Greater, Ordering::Equal) => {
				// player is straight right
				// 6 4  2
				// 8 ├┤◀▶
				// 7 5  3
				if Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}

				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
			},
			(Ordering::Greater, Ordering::Greater) => {
				// player is below right
				// 8 7  5
				// 6 ├┤ 3
				// 4 2 ◀▶
				if Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}

				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
			},
			(Ordering::Greater, Ordering::Less) => {
				// player is above right
				// 4 2 ◀▶
				// 6 ├┤ 3
				// 8 7  5
				if Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}

				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
			},
			(Ordering::Less, Ordering::Greater) => {
				// player is below left
				// 4 6  8
				// 2 ├┤ 7
				// ◀▶ 3 5
				if Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}

				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&left_top) && Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
			},
			(Ordering::Less, Ordering::Less) => {
				// player is above left
				// ◀▶ 3 5
				// 2 ├┤ 7
				// 4 6  8
				if Self::is_walkable_tile(&board[left_top]) || !check_tiles {
					result.push(left_top);
				}

				if !result.contains(&left_middle) && Self::is_walkable_tile(&board[left_middle]) || !check_tiles {
					result.push(left_middle);
				}
				if !result.contains(&middle_top) && Self::is_walkable_tile(&board[middle_top]) || !check_tiles {
					result.push(middle_top);
				}
				if !result.contains(&left_bottom) && Self::is_walkable_tile(&board[left_bottom]) || !check_tiles {
					result.push(left_bottom);
				}
				if !result.contains(&right_top) && Self::is_walkable_tile(&board[right_top]) || !check_tiles {
					result.push(right_top);
				}
				if !result.contains(&middle_bottom) && Self::is_walkable_tile(&board[middle_bottom]) || !check_tiles {
					result.push(middle_bottom);
				}
				if !result.contains(&right_middle) && Self::is_walkable_tile(&board[right_middle]) || !check_tiles {
					result.push(right_middle);
				}
				if !result.contains(&right_bottom) && Self::is_walkable_tile(&board[right_bottom]) || !check_tiles {
					result.push(right_bottom);
				}
			},
			(Ordering::Equal, Ordering::Equal) => {
				// Player is at the same position.
				unreachable!("The beast can't be at the same place at the player")
			},
		};

		result
	}

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
	fn astar(board: &Board, start: Coord, goal: &Coord) -> Option<Vec<Coord>> {
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

			for neighbor in Self::get_walkable_coords(board, &current, goal, true) {
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

		None
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::time::Instant;

	struct DummyBeast;

	impl Beast for DummyBeast {
		fn new(_position: Coord) -> Self {
			DummyBeast
		}

		fn advance(&mut self, _board: &mut Board, _player_position: Coord) -> BeastAction {
			BeastAction::Moved
		}

		fn get_score() -> u16 {
			0
		}
	}

	#[test]
	fn is_walkable_empty_tile_test() {
		assert!(DummyBeast::is_walkable_tile(&Tile::Empty), "Tile::Empty should be walkable");
	}

	#[test]
	fn is_walkable_player_tile_test() {
		assert!(DummyBeast::is_walkable_tile(&Tile::Player), "Tile::Player should be walkable");
	}

	#[test]
	fn is_walkable_blocked_tile_test() {
		assert!(!DummyBeast::is_walkable_tile(&Tile::Block), "Tile::Block should not be walkable");
		assert!(!DummyBeast::is_walkable_tile(&Tile::StaticBlock), "Tile::StaticBlock should not be walkable");
		assert!(!DummyBeast::is_walkable_tile(&Tile::CommonBeast), "Tile::CommonBeast should not be walkable");
		assert!(!DummyBeast::is_walkable_tile(&Tile::SuperBeast), "Tile::SuperBeast should not be walkable");
		assert!(!DummyBeast::is_walkable_tile(&Tile::Egg(Instant::now())), "Tile::Egg should not be walkable");
		assert!(
			!DummyBeast::is_walkable_tile(&Tile::EggHatching(Instant::now())),
			"Tile::EggHatching should not be walkable"
		);
		assert!(!DummyBeast::is_walkable_tile(&Tile::HatchedBeast), "Tile::HatchedBeast should not be walkable");
	}

	#[test]
	fn get_walkable_coords_below_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 5, row: 7 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 4 }, // middle_top
			],
			"Player straight below should yield expected neighbor order"
		);

		board[Coord { column: 5, row: 6 }] = Tile::StaticBlock;
		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::StaticBlock;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 4 }, // right_top
			],
			"Player straight below should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 4 }, // middle_top
			],
			"Player straight below should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player straight below should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_above_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 5, row: 3 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 6 }, // middle_bottom
			],
			"Player straight above should yield expected neighbor order"
		);

		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 4 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 6 }, // left_bottom
			],
			"Player straight above should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 6 }, // middle_bottom
			],
			"Player straight above should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player straight above should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_left_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 3, row: 5 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 6, row: 5 }, // right_middle
			],
			"Player straight left should yield expected neighbor order"
		);

		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 4 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 6, row: 5 }, // right_middle
			],
			"Player straight left should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 6, row: 5 }, // right_middle
			],
			"Player straight left should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player straight left should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_right_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 7, row: 5 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 4, row: 5 }, // left_middle
			],
			"Player straight right should yield expected neighbor order"
		);

		board[Coord { column: 5, row: 4 }] = Tile::StaticBlock;
		board[Coord { column: 5, row: 6 }] = Tile::StaticBlock;
		board[Coord { column: 6, row: 6 }] = Tile::StaticBlock;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 4, row: 5 }, // left_middle
			],
			"Player straight right should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 4, row: 5 }, // left_middle
			],
			"Player straight right should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player straight right should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_below_right_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 7, row: 7 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 4 }, // left_top
			],
			"Player below right should yield expected neighbor order"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 4 }, // middle_top
			],
			"Player below right should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 4 }, // left_top
			],
			"Player below right should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player below right should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_above_right_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 7, row: 3 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 6 }, // left_bottom
			],
			"Player above right should yield expected neighbor order"
		);

		board[Coord { column: 6, row: 4 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 6 }, // left_bottom
			],
			"Player above right should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 6 }, // left_bottom
			],
			"Player above right should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player above right should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_below_left_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 3, row: 7 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 4 }, // right_top
			],
			"Player below left should yield expected neighbor order"
		);

		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 5, row: 4 }, // middle_top
			],
			"Player below left should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 6, row: 6 }, // right_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 4 }, // right_top
			],
			"Player below left should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player below left should yield no neighbor for a blocked board"
		);
	}

	#[test]
	fn get_walkable_coords_above_left_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 3, row: 3 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 6 }, // right_bottom
			],
			"Player above left should yield expected neighbor order"
		);

		board[Coord { column: 4, row: 5 }] = Tile::StaticBlock;
		board[Coord { column: 4, row: 6 }] = Tile::StaticBlock;
		board[Coord { column: 5, row: 6 }] = Tile::StaticBlock;
		board[Coord { column: 6, row: 6 }] = Tile::StaticBlock;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 6, row: 5 }, // right_middle
			],
			"Player above left should yield expected neighbor order with blocking tiles"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 4, row: 4 }, // left_top
				Coord { column: 4, row: 5 }, // left_middle
				Coord { column: 5, row: 4 }, // middle_top
				Coord { column: 4, row: 6 }, // left_bottom
				Coord { column: 6, row: 4 }, // right_top
				Coord { column: 5, row: 6 }, // middle_bottom
				Coord { column: 6, row: 5 }, // right_middle
				Coord { column: 6, row: 6 }, // right_bottom
			],
			"Player above left should yield expected neighbor order without checking for blocking tiles"
		);

		board[Coord { column: 4, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 6, row: 4 }] = Tile::Block;
		board[Coord { column: 4, row: 5 }] = Tile::Block;
		board[Coord { column: 6, row: 5 }] = Tile::Block;
		board[Coord { column: 4, row: 6 }] = Tile::Block;
		board[Coord { column: 5, row: 6 }] = Tile::Block;
		board[Coord { column: 6, row: 6 }] = Tile::Block;

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"Player above left should yield no neighbor for a blocked board"
		);
	}

	#[test]
	#[should_panic]
	fn get_walkable_coords_same_pos_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 5, row: 5 };

		DummyBeast::get_walkable_coords(&board, &pos, &player, true);
	}

	#[test]
	fn get_walkable_coords_without_tile_check_test() {
		// Create a board where all tiles are blocked.
		let board = Board::new([[Tile::Block; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 5, row: 5 };
		let player = Coord { column: 5, row: 7 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, false),
			vec![
				Coord { column: 5, row: 6 },
				Coord { column: 4, row: 6 },
				Coord { column: 6, row: 6 },
				Coord { column: 4, row: 5 },
				Coord { column: 6, row: 5 },
				Coord { column: 4, row: 4 },
				Coord { column: 6, row: 4 },
				Coord { column: 5, row: 4 },
			],
			"When check_tiles is false, all neighbor coordinates are returned regardless of block state"
		);

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			Vec::new(),
			"When check_tiles is true, no neighbor coordinates are returned for a full board"
		);
	}

	#[test]
	fn get_walkable_coords_boundary_top_left_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord { column: 0, row: 0 };
		let player = Coord { column: 0, row: 1 };

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord { column: 0, row: 1 }, // middle_bottom
				Coord { column: 1, row: 1 }, // right_bottom
				Coord { column: 0, row: 0 }, // left_middle
				Coord { column: 1, row: 0 }, // right_middle
			],
			"Boundary test: Top-left corner should properly clamp coordinates without duplicates"
		);
	}

	#[test]
	fn get_walkable_coords_boundary_top_right_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord {
			column: BOARD_WIDTH - 1,
			row: 0,
		};
		let player = Coord {
			column: BOARD_WIDTH - 1,
			row: 1,
		};

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord {
					column: BOARD_WIDTH - 1,
					row: 1,
				},
				Coord {
					column: BOARD_WIDTH - 2,
					row: 1,
				},
				Coord {
					column: BOARD_WIDTH - 2,
					row: 0,
				},
				Coord {
					column: BOARD_WIDTH - 1,
					row: 0,
				},
			],
			"Boundary test: Top-right corner should properly clamp coordinates and remove duplicates"
		);
	}

	#[test]
	fn get_walkable_coords_boundary_bottom_left_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord {
			column: 0,
			row: BOARD_HEIGHT - 1,
		};
		let player = Coord {
			column: 0,
			row: BOARD_HEIGHT - 2,
		};

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord {
					column: 0,
					row: BOARD_HEIGHT - 2,
				},
				Coord {
					column: 1,
					row: BOARD_HEIGHT - 2,
				},
				Coord {
					column: 0,
					row: BOARD_HEIGHT - 1,
				},
				Coord {
					column: 1,
					row: BOARD_HEIGHT - 1,
				},
			],
			"Boundary test: Bottom-left corner should properly clamp coordinates and remove duplicates"
		);
	}

	#[test]
	fn get_walkable_coords_boundary_bottom_right_test() {
		let board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let pos = Coord {
			column: BOARD_WIDTH - 1,
			row: BOARD_HEIGHT - 1,
		};
		let player = Coord {
			column: BOARD_WIDTH - 1,
			row: BOARD_HEIGHT - 2,
		};

		assert_eq!(
			DummyBeast::get_walkable_coords(&board, &pos, &player, true),
			vec![
				Coord {
					column: BOARD_WIDTH - 1,
					row: BOARD_HEIGHT - 2,
				},
				Coord {
					column: BOARD_WIDTH - 2,
					row: BOARD_HEIGHT - 2,
				},
				Coord {
					column: BOARD_WIDTH - 2,
					row: BOARD_HEIGHT - 1,
				},
				Coord {
					column: BOARD_WIDTH - 1,
					row: BOARD_HEIGHT - 1,
				},
			],
			"Boundary test: Bottom-right corner should properly clamp coordinates and remove duplicates"
		);
	}

	#[test]
	fn heuristic_test() {
		let a = Coord { column: 3, row: 4 };
		let b = Coord { column: 6, row: 8 };
		// max(|6-3|, |8-4|) = max(3, 4) = 4
		assert_eq!(DummyBeast::heuristic(&a, &b), 4, "We have calculated the heuristic distance between two coordinates");

		let a = Coord { column: 5, row: 5 };
		let b = Coord { column: 2, row: 7 };
		// max(|2-5|, |7-5|) = max(3, 2) = 3
		assert_eq!(DummyBeast::heuristic(&a, &b), 3, "We have calculated the heuristic distance between two coordinates");
	}

	#[test]
	fn reconstruct_path_test() {
		let mut came_from = HashMap::new();
		let start = Coord { column: 1, row: 1 };
		let mid1 = Coord { column: 2, row: 1 };
		let mid2 = Coord { column: 3, row: 2 };
		let goal = Coord { column: 3, row: 3 };

		came_from.insert(mid1, start);
		came_from.insert(mid2, mid1);
		came_from.insert(goal, mid2);

		let path = DummyBeast::reconstruct_path(&came_from, goal);
		assert_eq!(path, vec![start, mid1, mid2, goal], "We have reconstructed the path from start to goal");
	}

	#[test]
	fn astar_direct_path_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		let path = DummyBeast::astar(&board, beast_position, &player_position);

		assert!(path.is_some(), "Path should be calculated");
		let unwrapped_path = path.unwrap();
		assert_eq!(unwrapped_path[0], beast_position, "Path should start from the beast's position");
		assert_eq!(unwrapped_path.last().unwrap(), &player_position, "Path should end at the player's position");
	}

	#[test]
	fn astar_path_around_obstacle_test() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let beast_position = Coord { column: 0, row: 0 };
		let player_position = Coord { column: 4, row: 4 };

		board[beast_position] = Tile::SuperBeast;
		board[player_position] = Tile::Player;

		// partial wall
		for row in 0..3 {
			board[Coord { column: 2, row }] = Tile::StaticBlock;
		}

		let path = DummyBeast::astar(&board, beast_position, &player_position);

		assert!(path.is_some(), "A* should find a path around the wall");
		let unwrapped_path = path.unwrap();
		assert_eq!(unwrapped_path[0], beast_position, "Path should start from the beast's position");
		assert_eq!(unwrapped_path.last().unwrap(), &player_position, "Path should end at the player's position");

		// Verify that the path goes around the obstacle (no coordinates with column = 2)
		for coord in &unwrapped_path {
			if coord.row < 3 {
				assert_ne!(coord.column, 2, "Path should not go through the obstacle");
			}
		}
	}
}
