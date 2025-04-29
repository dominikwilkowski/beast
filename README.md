![The beast game play](/assets/gameplay.gif)

> BEAST is a homage to the 1984 ASCII game "[BEAST](https://en.wikipedia.org/wiki/Beast_(video_game))"
> from Dan Baker, Alan Brown, Mark Hamilton and Derrick Shadel.

# Beast

- [How to install](#how-to-install)
- [How to play](#how-to-play)
- [Global highscore](#global-highscore)
- [Differences](#differences)
- [Contributing](#contributing)
- [Test](#test)
- [Release History](#release-history)
- [License](#license)


## How to install

> [!NOTE]
> This tool requires a [POSIX](https://en.wikipedia.org/wiki/POSIX)-compatible terminal (i.e. `stty`-style mode toggling) and only runs on Unix-like systems.
> Windows is not supported right now.

### Homebrew

Install `beast` via [Homebrew](https://brew.sh/):

```sh
brew tap dominikwilkowski/beast https://github.com/dominikwilkowski/beast.git
brew install dominikwilkowski/beast/beast
beast # play the game
```

### Cargo

Install `beast` via [Cargo](https://doc.rust-lang.org/cargo/):

```sh
cargo install beast1984
beast # play the game
```

### Build from source

You can build `beast` from source by cloning the repository and running:

```sh
git clone https://github.com/dominikwilkowski/beast.git
cd beast
cargo build --release
```

## How to play

![The help of the game](/assets/help.gif)

The object of this arcade-like game is to survive through a number of levels
while crushing the beasts (`├┤`) with movable blocks (`░░`).
The beasts are attracted to the player's (`◄►`) position every move.
The beginning levels have only the common beasts, however in later levels
the more challenging super-beasts appear (`╟╢`).
These super-beasts are harder to kill as they must be crushed against a
static block (`▓▓`).
At levels beyond, the eggs (`○○`) are introduced, implying greater challenge.
These enemies are dormant at the beginning of each level, but will in time hatch
into a hatched beast (`╬╬`).
These beasts are the hardest to kill, as they can also move blocks to crush the
player.
They can however be killed as easily as the regular beasts, against any object.

## Global highscore

![The global highscore](/assets/highscore.gif)

The global highscore is synced with an online server.
This is where you can enter yourself to compete with others world wide.
The top 100 are saved until someone better comes around.

## Differences

I've attempted to keep this game as close to the original as practical.
Whever possible I copied elements straight and in other areas I tried to guess as close to what the original code would have done.
The level config is an approximation to the best of my abilities.

There are some differences though which I like to highlight:
```diff
- Board size: 38x21 tiles
+ Board size: 50x30 tiles

- Pull blocks while holding the spacebar

+ A global highscore synced with an online server

- `HatchedBeasts` can squish other beasts when moving blocks
+ `HatchedBeasts` can't squish other beasts when moving blocks

- There are 3 x "EASY", 4 x "NOVICE", 4 x "HARD", 4 x "UNKNOWN", 4 x "ADVANCED", 4 x "EXPERT" and 3 x "PRO" levels, 
+ There are exactly 10 predefined levels (for now)

- The footer tells you what keys you can push when playing

+ The game now has a help screen with pagination

- As time goes by, the game will speed up, and the Beasts will also speed up

+ The pathfinding algorithm is an more advanced and efficient than the original

- There are EXPLOSIVE BLOCKS

+ There is a logo at the top

- Beast has a multiplayer mode (on the same keyboard)

+ Added an background color animation when squishing a beast or when player dies
```

## Contributing

If you want to contribute to this project, please create a pull request and
make sure you make the tests pass and run `cargo fmt`.

## Test

All tests are run via `cargo test` and are extensively documented.

## Release History
* 1.0.1  -  Removed beast_common create
* 1.0.0  -  First rust release

## License
Copyright (c) Dominik Wilkowski.
Licensed under the [GNU GPLv3](https://github.com/dominikwilkowski/beast/blob/main/LICENSE).
