//! this module contains the player struct which manages the player movements

use rand::Rng;

use crate::{
	BOARD_HEIGHT, BOARD_WIDTH, Coord, Dir, Tile,
	beasts::{Beast, CommonBeast, Egg, HatchedBeast, SuperBeast},
	board::Board,
};

/// actions a player can take
pub enum PlayerAction {
	/// killed a common beast
	KillCommonBeast(Coord),
	/// killed a super beast
	KillSuperBeast(Coord),
	/// killed an egg
	KillEgg(Coord),
	/// killed a hatched beast
	KillHatchedBeast(Coord),
	/// player was killed
	KillPlayer,
	/// no action taken
	None,
}

pub struct Player {
	pub position: Coord,
	pub lives: u8,
	pub score: u16,
	pub beasts_killed: u16,
}

impl Player {
	pub fn new(position: Coord) -> Self {
		Self {
			position,
			lives: 5,
			score: 0,
			beasts_killed: 0,
		}
	}

	fn get_next_coord(coord: Coord, dir: &Dir) -> Option<Coord> {
		match dir {
			Dir::Up if coord.row > 0 => Some(Coord {
				row: coord.row - 1,
				column: coord.column,
			}),
			Dir::Right if coord.column < BOARD_WIDTH - 1 => Some(Coord {
				row: coord.row,
				column: coord.column + 1,
			}),
			Dir::Down if coord.row < BOARD_HEIGHT - 1 => Some(Coord {
				row: coord.row + 1,
				column: coord.column,
			}),
			Dir::Left if coord.column > 0 => Some(Coord {
				row: coord.row,
				column: coord.column - 1,
			}),
			_ => None,
		}
	}

	pub fn advance(&mut self, board: &mut Board, dir: &Dir) -> PlayerAction {
		if let Some(new_coord) = Self::get_next_coord(self.position, dir) {
			match board[new_coord] {
				Tile::Empty => {
					board[new_coord] = Tile::Player;
					board[self.position] = Tile::Empty;
					self.position = new_coord;
					PlayerAction::None
				},
				Tile::Block => {
					let mut next_tile = Tile::Block;
					let mut prev_coord = new_coord;

					while next_tile == Tile::Block {
						if let Some(next_coord) = Self::get_next_coord(prev_coord, dir) {
							next_tile = board[next_coord];

							match next_tile {
								Tile::Block => {
									// we need to seek deeper into the stack to find the end of this Block chain (pun not intended)
									// so nothing needs to be done here and the while loop with continue
								},
								Tile::CommonBeast | Tile::HatchedBeast | Tile::Egg(_) | Tile::EggHatching(_) => {
									// can be squished against the frame of the board
									if Self::get_next_coord(next_coord, dir)
										.is_none_or(|coord| board[coord] == Tile::Block || board[coord] == Tile::StaticBlock)
									{
										self.beasts_killed += 1;

										board[self.position] = Tile::Empty;
										board[new_coord] = Tile::Player;
										self.position = new_coord;
										board[next_coord] = Tile::Block;

										match next_tile {
											Tile::CommonBeast => {
												self.score += CommonBeast::get_score();
												return PlayerAction::KillCommonBeast(next_coord);
											},
											Tile::Egg(_) | Tile::EggHatching(_) => {
												self.score += Egg::get_score();
												return PlayerAction::KillEgg(next_coord);
											},
											Tile::HatchedBeast => {
												self.score += HatchedBeast::get_score();
												return PlayerAction::KillHatchedBeast(next_coord);
											},
											_ => {
												unreachable!("No other tiles can be found in this match arm")
											},
										}
									}
								},
								Tile::SuperBeast => {
									// can't be squished against the frame of the board
									if Self::get_next_coord(next_coord, dir).is_some_and(|coord| board[coord] == Tile::StaticBlock) {
										self.beasts_killed += 1;

										board[self.position] = Tile::Empty;
										board[new_coord] = Tile::Player;
										self.position = new_coord;
										board[next_coord] = Tile::Block;
										self.score += SuperBeast::get_score();

										return PlayerAction::KillSuperBeast(next_coord);
									}
								},
								Tile::StaticBlock | Tile::Player => {
									// nothing happens on this move since the user is trying to push a stack of blocks against a StaticBlock | Player
									return PlayerAction::None;
								},
								Tile::Empty => {
									board[self.position] = Tile::Empty;
									board[new_coord] = Tile::Player;
									self.position = new_coord;
									board[next_coord] = Tile::Block;

									return PlayerAction::None;
								},
							}

							prev_coord = next_coord;
						} else {
							return PlayerAction::None;
						}
					}
					PlayerAction::None
				},
				Tile::CommonBeast | Tile::SuperBeast | Tile::HatchedBeast => {
					self.lives -= 1;
					self.respawn(board);
					PlayerAction::KillPlayer
				},
				Tile::Egg(_) | Tile::EggHatching(_) | Tile::StaticBlock | Tile::Player => {
					/* nothing happens */
					PlayerAction::None
				},
			}
		} else {
			PlayerAction::None
		}
	}

	pub fn respawn(&mut self, board: &mut Board) {
		let mut rng = rand::rng();
		let old_coord = self.position;
		let new_coord = loop {
			let coord = Coord {
				column: rng.random_range(0..BOARD_WIDTH),
				row: rng.random_range(0..BOARD_HEIGHT),
			};

			if board[coord] == Tile::Empty {
				break coord;
			}
		};

		board[new_coord] = Tile::Player;
		if board[old_coord] == Tile::Player {
			board[old_coord] = Tile::Empty;
		}
		self.position = new_coord;
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::time::Instant;

	#[test]
	fn moving() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 10 });

		// *************
		// MOVING UP
		// *************
		board[Coord { column: 5, row: 10 }] = Tile::Player;

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 9 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 9 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 0 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 0 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 0 }, "Player should not have moved");
		assert_eq!(board[Coord { column: 5, row: 0 }], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING RIGHT
		// *************
		board[Coord { column: 5, row: 0 }] = Tile::Empty;
		board[Coord {
			column: BOARD_WIDTH - 5,
			row: 10,
		}] = Tile::Player;
		player.position = Coord {
			row: 10,
			column: BOARD_WIDTH - 5,
		};

		player.advance(&mut board, &Dir::Right);
		assert_eq!(
			player.position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 4
			},
			"Player should move right one column"
		);
		assert_eq!(
			board[Coord {
				column: BOARD_WIDTH - 4,
				row: 10
			}],
			Tile::Player,
			"Player tile should be placed at new position"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		assert_eq!(
			player.position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 1
			},
			"Player should move right one column"
		);
		assert_eq!(
			board[Coord {
				column: BOARD_WIDTH - 1,
				row: 10
			}],
			Tile::Player,
			"Player tile should be placed at new position"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		assert_eq!(
			player.position,
			Coord {
				row: 10,
				column: BOARD_WIDTH - 1
			},
			"Player should not have moved"
		);
		assert_eq!(
			board[Coord {
				column: BOARD_WIDTH - 1,
				row: 10
			}],
			Tile::Player,
			"Player tile should not have moved"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING DOWN
		// *************
		board[Coord {
			column: BOARD_WIDTH - 1,
			row: 10,
		}] = Tile::Empty;
		board[Coord {
			column: 5,
			row: BOARD_HEIGHT - 3,
		}] = Tile::Player;
		player.position = Coord {
			row: BOARD_HEIGHT - 3,
			column: 5,
		};

		player.advance(&mut board, &Dir::Down);
		assert_eq!(
			player.position,
			Coord {
				row: BOARD_HEIGHT - 2,
				column: 5
			},
			"Player should move down one row"
		);
		assert_eq!(
			board[Coord {
				column: 5,
				row: BOARD_HEIGHT - 2
			}],
			Tile::Player,
			"Player tile should be placed at new position"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Down);
		assert_eq!(
			player.position,
			Coord {
				row: BOARD_HEIGHT - 1,
				column: 5
			},
			"Player should move down one row"
		);
		assert_eq!(
			board[Coord {
				column: 5,
				row: BOARD_HEIGHT - 1
			}],
			Tile::Player,
			"Player tile should be placed at new position"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		assert_eq!(
			player.position,
			Coord {
				row: BOARD_HEIGHT - 1,
				column: 5
			},
			"Player should not have moved"
		);
		assert_eq!(
			board[Coord {
				column: 5,
				row: BOARD_HEIGHT - 1
			}],
			Tile::Player,
			"Player tile should not have moved"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		// *************
		// MOVING LEFT
		// *************
		board[Coord {
			column: 5,
			row: BOARD_HEIGHT - 1,
		}] = Tile::Empty;
		board[Coord { column: 5, row: 10 }] = Tile::Player;
		player.position = Coord { column: 5, row: 10 };
		board[Coord { column: 5, row: 10 }] = Tile::Player;

		player.advance(&mut board, &Dir::Left);
		assert_eq!(player.position, Coord { column: 4, row: 10 }, "Player should move left one column");
		assert_eq!(board[Coord { column: 4, row: 10 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		assert_eq!(player.position, Coord { column: 0, row: 10 }, "Player should move left one column");
		assert_eq!(board[Coord { column: 0, row: 10 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);

		player.advance(&mut board, &Dir::Left);
		assert_eq!(player.position, Coord { column: 0, row: 10 }, "Player should not have moved");
		assert_eq!(board[Coord { column: 0, row: 10 }], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
	}

	#[test]
	fn push_block() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });
		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::Block;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ░░
		// 4 ▌
		// 5 ▌        ◄►

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The Block hasn't moved");

		// 1 ▌
		// 2 ▌
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 3 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Block, "The Block has moved up one row");

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ◄►
		// 4 ▌
		// 5 ▌

		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Left);
		assert_eq!(player.position, Coord { column: 5, row: 2 }, "Player should moved right, up and left");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 4, row: 2 }], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌      ░░◄►
		// 3 ▌
		// 4 ▌
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Down);
		assert_eq!(player.position, Coord { column: 4, row: 2 }, "Player should moved up, left and down");
		assert_eq!(board[Coord { column: 4, row: 2 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 4, row: 3 }], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌      ◄►
		// 3 ▌      ░░
		// 4 ▌
		// 5 ▌

		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Right);
		assert_eq!(player.position, Coord { column: 4, row: 3 }, "Player should moved left, down and right");
		assert_eq!(board[Coord { column: 4, row: 3 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The Block has moved left");

		// 1 ▌
		// 2 ▌
		// 3 ▌      ◄►░░
		// 4 ▌
		// 5 ▌
	}

	#[test]
	fn push_block_chain() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 0, row: 10 });

		board[Coord { column: 0, row: 10 }] = Tile::Player;
		board[Coord { column: 0, row: 9 }] = Tile::Block;
		board[Coord { column: 0, row: 8 }] = Tile::Block;
		board[Coord { column: 0, row: 7 }] = Tile::Block;
		board[Coord { column: 0, row: 6 }] = Tile::Block;
		board[Coord { column: 0, row: 5 }] = Tile::Empty;
		board[Coord { column: 0, row: 4 }] = Tile::StaticBlock;

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌░░
		// 10 ▌◄►

		// move up
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 0, row: 9 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 0, row: 9 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board[Coord { column: 0, row: 8 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 7 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 6 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 5 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 4 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌░░
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌◄►
		// 10 ▌

		// move up again
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 0, row: 9 }, "Player should not move");
		assert_eq!(board[Coord { column: 0, row: 9 }], Tile::Player, "Player tile should not move");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board[Coord { column: 0, row: 8 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 7 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 6 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 5 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 4 }], Tile::StaticBlock, "The StaticBlock should not move");

		//    ▛▀
		//  0 ▌
		//  1 ▌
		//  2 ▌
		//  3 ▌
		//  4 ▌▓▓
		//  5 ▌░░
		//  6 ▌░░
		//  7 ▌░░
		//  8 ▌░░
		//  9 ▌◄►
		// 10 ▌

		// now let's cheat and remove the static block
		board[Coord { column: 0, row: 4 }] = Tile::Empty;
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 0, row: 4 }, "Player should move up four rows");
		assert_eq!(board[Coord { column: 0, row: 9 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 0, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board[Coord { column: 0, row: 3 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 2 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 1 }], Tile::Block, "The Blocks should have moved up");
		assert_eq!(board[Coord { column: 0, row: 0 }], Tile::Block, "The Blocks should have moved up");

		//    ▛▀
		//  0 ▌░░
		//  1 ▌░░
		//  2 ▌░░
		//  3 ▌░░
		//  4 ▌◄►
		//  5 ▌
		//  6 ▌
		//  7 ▌
		//  8 ▌
		//  9 ▌
		// 10 ▌

		// now that we're up against the wall let's move up one more time
		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 0, row: 4 }, "Player should not move");
		assert_eq!(board[Coord { column: 0, row: 4 }], Tile::Player, "Player tile should not move");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			4,
			"There should be exactly four block tiles"
		);
		assert_eq!(board[Coord { column: 0, row: 3 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 2 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 1 }], Tile::Block, "The Blocks should not move");
		assert_eq!(board[Coord { column: 0, row: 0 }], Tile::Block, "The Blocks should not move");

		//    ▛▀
		//  0 ▌░░
		//  1 ▌░░
		//  2 ▌░░
		//  3 ▌░░
		//  4 ▌◄►
		//  5 ▌
		//  6 ▌
		//  7 ▌
		//  8 ▌
		//  9 ▌
		// 10 ▌
	}

	#[test]
	fn push_static_block() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });

		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::StaticBlock;

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌
		// 5 ▌        ◄►

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should not have moved");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should not have moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		assert_eq!(player.position, Coord { column: 6, row: 3 }, "Player should now be next to the StaticBlock");
		assert_eq!(board[Coord { column: 6, row: 3 }], Tile::Player, "Player tile should have moved to the right and up");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌        ▓▓◄►
		// 4 ▌
		// 5 ▌

		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Up);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Down);
		assert_eq!(player.position, Coord { column: 5, row: 2 }, "Player should now be above the StaticBlock");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Player, "Player tile should have moved up and left");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌        ◄►
		// 3 ▌        ▓▓
		// 4 ▌
		// 5 ▌

		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Left);
		player.advance(&mut board, &Dir::Down);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		player.advance(&mut board, &Dir::Right);
		assert_eq!(player.position, Coord { column: 4, row: 3 }, "Player should now be above the StaticBlock");
		assert_eq!(board[Coord { column: 4, row: 3 }], Tile::Player, "Player tile should have moved up and left");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::StaticBlock, "The StaticBlock hasn't moved");

		// 2 ▌
		// 3 ▌      ◄►▓▓
		// 4 ▌
		// 5 ▌
	}

	#[test]
	fn squish_common_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });

		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 3 }] = Tile::CommonBeast;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ├┤
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 5 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::CommonBeast, "The beast hasn't moved");
		assert_eq!(player.beasts_killed, 0, "The player has not killed any beasts yet");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			1,
			"There should be exactly one beast tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);

		// now let's place a block behind the beast
		board[Coord { column: 5, row: 2 }] = Tile::Block;

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ├┤
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The block has moved up");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Block, "The other block hasn't moved");
		assert_eq!(player.beasts_killed, 1, "The player has killed one beast");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			0,
			"There should be exactly zero beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			2,
			"There should be exactly two block tiles"
		);

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌
		// 6 ▌
	}

	#[test]
	fn squish_egg() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });

		let egg = Tile::Egg(Instant::now());
		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 3 }] = egg;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ○○
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 5 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], egg, "The egg hasn't moved");
		assert_eq!(player.beasts_killed, 0, "The player has not killed any beasts yet");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == egg).count(),
			1,
			"There should be exactly one egg tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);

		// now let's place a block behind the beast
		board[Coord { column: 5, row: 2 }] = Tile::Block;

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ○○
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The block has moved up");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Block, "The other block hasn't moved");
		assert_eq!(player.beasts_killed, 1, "The player has killed one beast");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == egg).count(),
			0,
			"There should be exactly zero egg tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			2,
			"There should be exactly two block tiles"
		);

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌
		// 6 ▌
	}

	#[test]
	fn squish_hatched_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });

		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 3 }] = Tile::HatchedBeast;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ╬╬
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 5 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::HatchedBeast, "The hatched beast hasn't moved");
		assert_eq!(player.beasts_killed, 0, "The player has not killed any beasts yet");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);

		// now let's place a block behind the beast
		board[Coord { column: 5, row: 2 }] = Tile::Block;

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ╬╬
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The block has moved up");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Block, "The other block hasn't moved");
		assert_eq!(player.beasts_killed, 1, "The player has killed one beast");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			0,
			"There should be exactly zero hatched beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			2,
			"There should be exactly two block tiles"
		);

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌
		// 6 ▌
	}

	#[test]
	fn squish_super_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 5 });

		board[Coord { column: 5, row: 5 }] = Tile::Player;
		board[Coord { column: 5, row: 4 }] = Tile::Block;
		board[Coord { column: 5, row: 3 }] = Tile::SuperBeast;

		// 1 ▌
		// 2 ▌
		// 3 ▌        ╟╢
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 5 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::SuperBeast, "The super beast hasn't moved");
		assert_eq!(player.beasts_killed, 0, "The player has not killed any beasts yet");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			1,
			"There should be exactly one super beast tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tile"
		);

		// now let's place a block behind the beast
		board[Coord { column: 5, row: 2 }] = Tile::Block;

		// 1 ▌
		// 2 ▌        ░░
		// 3 ▌        ╟╢
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		assert_eq!(player.position, Coord { column: 5, row: 5 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Block, "The block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::Block, "The other block hasn't moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::SuperBeast, "The super beast hasn't moved");
		assert_eq!(player.beasts_killed, 0, "The player has not killed any beasts yet");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			1,
			"There should be exactly one super beast tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			2,
			"There should be exactly two block tiles"
		);

		// now let's place a static block behind the beast
		board[Coord { column: 5, row: 2 }] = Tile::StaticBlock;

		// 1 ▌
		// 2 ▌        ▓▓
		// 3 ▌        ╟╢
		// 4 ▌        ░░
		// 5 ▌        ◄►
		// 6 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should move up one row");
		assert_eq!(board[Coord { column: 5, row: 5 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player tile should be placed at new position");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::Block, "The block has moved up");
		assert_eq!(board[Coord { column: 5, row: 2 }], Tile::StaticBlock, "The static block hasn't moved");
		assert_eq!(player.beasts_killed, 1, "The player has killed one beast");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			0,
			"There should be exactly zero super beast tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Block).count(),
			1,
			"There should be exactly one block tiles"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::StaticBlock).count(),
			1,
			"There should be exactly one static block tile"
		);

		// 1 ▌
		// 2 ▌        ▓▓
		// 3 ▌        ░░
		// 4 ▌        ◄►
		// 5 ▌
		// 6 ▌
	}

	#[test]
	fn getting_killed_by_common_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 4 });

		board[Coord { column: 5, row: 4 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::CommonBeast;

		assert_eq!(player.lives, 5, "The player starts with 5 lives");

		// 2 ▌
		// 3 ▌        ├┤
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::CommonBeast, "Beast has not moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::CommonBeast).count(),
			1,
			"There should be exactly one common beast tile"
		);
		assert_eq!(player.lives, 4, "The player has lost a live");
	}

	#[test]
	fn getting_killed_by_super_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 4 });

		board[Coord { column: 5, row: 4 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::SuperBeast;

		assert_eq!(player.lives, 5, "The player starts with 5 lives");

		// 2 ▌
		// 3 ▌        ╟╢
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::SuperBeast, "Beast has not moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::SuperBeast).count(),
			1,
			"There should be exactly one super beast tile"
		);
		assert_eq!(player.lives, 4, "The player has lost a live");
	}

	#[test]
	fn getting_killed_by_hatched_beast() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 4 });

		board[Coord { column: 5, row: 4 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = Tile::HatchedBeast;

		assert_eq!(player.lives, 5, "The player starts with 5 lives");

		// 2 ▌
		// 3 ▌        ╬╬
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Empty, "Previous player tile should be empty now");
		assert_eq!(board[Coord { column: 5, row: 3 }], Tile::HatchedBeast, "Beast has not moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::HatchedBeast).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
		assert_eq!(player.lives, 4, "The player has lost a live");
	}

	#[test]
	fn not_getting_killed_by_egg() {
		let mut board = Board::new([[Tile::Empty; BOARD_WIDTH]; BOARD_HEIGHT]);
		let mut player = Player::new(Coord { column: 5, row: 4 });

		let egg = Tile::Egg(Instant::now());
		board[Coord { column: 5, row: 4 }] = Tile::Player;
		board[Coord { column: 5, row: 3 }] = egg;

		assert_eq!(player.lives, 5, "The player starts with 5 lives");

		// 2 ▌
		// 3 ▌        ○○
		// 4 ▌        ◄►
		// 5 ▌

		player.advance(&mut board, &Dir::Up);
		assert_eq!(player.position, Coord { column: 5, row: 4 }, "Player should not have moved up");
		assert_eq!(board[Coord { column: 5, row: 4 }], Tile::Player, "Player has not moved");
		assert_eq!(board[Coord { column: 5, row: 3 }], egg, "Egg has not moved");
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == Tile::Player).count(),
			1,
			"There should be exactly one player tile"
		);
		assert_eq!(
			board.data.iter().flatten().filter(|&&tile| tile == egg).count(),
			1,
			"There should be exactly one hatched beast tile"
		);
		assert_eq!(player.lives, 5, "The player has not lost a live");
	}
}
