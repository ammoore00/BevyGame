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

Each major world story should also have multiple outcomes and secrets, providing value to playing
through the same level multiple times. See [Level Architecture](notes/Level%20Architecture.md)

There should be some way to encourage multiple completions of the game as well.

## Setting

Some kind of steampunk magitech setting.

# License

This game and its components are released under a combination of the Mozilla Public License 2.0 (MPL 2.0)
and custom terms. This summary outlines what you can and cannot do with the game.

### What You Can Do:
- **Modify the Engine Code**: You can modify and redistribute the game engine code (located in the `src/` directory)
under the terms of the MPL 2.0.
- **Create User-Generated Content**: You can create, modify, and share your own content (e.g., mods, tools, assets, etc.)
for the game, even for commercial purposes, as long as you follow the rules below.
- **Build the Game Locally**: You can download the source code, build the game locally, and run it for private use.
### What You Cannot Do:
- **Redistribute Official Game Content**: You cannot redistribute, reuse, or share any of the game’s official assets
(e.g., images, music, levels, etc.) without explicit permission. This includes game assets that are not under MPL 2.0.
- **Redistribute the Official Content Generator**: You cannot redistribute the code used to generate official game
content (located in the `datagen/` directory) or any modified versions of it.
- **Distribute Compiled Builds with Official Content**: You cannot distribute compiled versions of the game that
include official game content unless you have explicit permission.
- **Imply Official Endorsement**: You can’t claim that your user-generated content is officially endorsed or
sponsored by the game, unless explicitly authorized.

### Third Party Content
Some third-party assets are included in the game, and these are subject to their own licenses. Be sure to follow
any third-party terms when using or redistributing them.

### Reminder
This summary is for quick reference and is not the authoritative license. Please read the full
[license document](LICENSE.md) for complete details on what’s allowed and not allowed.