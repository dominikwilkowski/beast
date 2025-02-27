```shell
  ╔╗  ╔═╗ ╔═╗ ╔═╗ ╔╦╗
  ╠╩╗ ║╣  ╠═╣ ╚═╗  ║
  ╚═╝ ╚═╝ ╩ ╩ ╚═╝  ╩
```

![The beast game](https://raw.githubusercontent.com/dominikwilkowski/beast.js/master/assets/play.gif)

> BEAST is a homage to the 1984 ASCII game "[BEAST](https://en.wikipedia.org/wiki/Beast_(video_game))"
> from Dan Baker, Alan Brown, Mark Hamilton and Derrick Shadel.

# Beast

- [How to install](#how-to-install)
- [How to play](#how-to-play)
- [Contributing](#contributing)
- [Test](#test)
- [Release History](#release-history)
- [License](#license)


## How to install

TODO

## How to play

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

## Contributing

If you want to contribute to this project, please create a pull request and
make sure you make the tests pass and run `cargo fmt`.

## Test

All tests are run via `cargo test` and are extensively documented.

## Release History
* 1.0.0  -  Ported to rust
* 0.1.5  -  Fixed dependencies
* 0.1.4  -  Fixed lives color bug
* 0.1.3  -  Improved drawing
* 0.1.2  -  Fixed error message
* 0.1.1  -  Added level indicator, fixed lives display
* 0.1.0  -  alpha release


## License
Copyright (c) Dominik Wilkowski.
Licensed under the [GNU GPLv3](https://github.com/dominikwilkowski/beast/blob/main/LICENSE).
