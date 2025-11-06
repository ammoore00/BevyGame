# Design Overview
This game is designed as a real-time action persistent roguelite.

## Real-time Action

3D isometric action combat game.

### Classes

The game will have classes which define abilities and perks.

Classes will be swappable mid-run.

### Armor

Armor will be made of entire sets, there will be no individual pieces.

Armor will have customizable upgrade slots which are class-agnostic perks.

Each armor set will have different stats, and different upgrades available.

### Weapons

Weapons will be separated into a few different classes.
Each weapon class will share the same character animation, but may have the weapon art change.

Special weapons will still use the weapon class system, but may have unique effects on top of it.

Weapons will also have customizable upgrade slots like armor which are class-agnostic.

## Persistent Roguelite:

Each level will be somewhat randomized like a traditional roguelite, but there is no permadeath.
Instead, there will be a bonfire mechanic for checkpoints.

### Randomized Level Layouts

Levels will consist of composable elements assembled into a full level layout.

### Replayability

Randomization of level components should give some inherent replayability.

Each major world story should also have multiple outcomes and secrets, providing value to playing through the same level multiple times.

There should be some way to encourage multiple completions of the game as well.

## Setting

Some kind of steampunk magitech setting.

# License
This project is licensed under the MPL 2.0 License - see the [license](LICENSE.md) file for details.