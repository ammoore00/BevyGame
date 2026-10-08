## Definitions

"Engine Code" and "Game Engine" refer to the code and build infrastructure used to load
and run content for the game. This includes all source code under `src/` and `crates/`
(excluding `crates/datagen/`), as well as project build manifests, toolchain settings,
configuration files, continuous integration (CI) scripts, and other build infrastructure
required for building the Engine Code (including `Cargo.toml`, `Cargo.lock`, `clippy.toml`,
and configuration under `.github/`).

"Official Game Content" refers to all game assets, data, and content provided by the
project, including images, audio, fonts, maps, levels, dialogue, characters,
configuration files, generated data files, source asset files, and other non-engine
content, except where otherwise stated.

The "Official Content Generator" refers to the code used to generate the game's
official game content, located under `crates/datagen/`.

"User Generated Content" refers to original mods, game content, and content
generation tools created by end users. User Generated Content does not include
unmodified Official Game Content, the Official Content Generator, or modified versions
of the Official Content Generator.

## Engine Code

All game engine code is licensed under the Mozilla Public License 2.0.
See [LICENSE_MPL_2.0.md](LICENSE_MPL_2.0.md) for details.

You may modify the Engine Code and redistribute your modifications under the terms of
the MPL 2.0. However, any redistribution of the game (or game binaries) that includes
Official Game Content or the Official Content Generator is not permitted without
explicit permission.

## Official Game Content

Official Game Content is All Rights Reserved, except where otherwise stated in
this file or in [LICENSE_THIRD_PARTY.md](LICENSE_THIRD_PARTY.md).

You are granted permission to capture, stream, and monetize video and audio footage of the game
(such as on Twitch or YouTube) provided you do not distribute the raw asset files.

The following assets required for basic engine functionality are exempt
and are provided under the terms of the MPL 2.0:
- _TODO_

## Official Content Generator

The Official Content Generator is made source-available, and is licensed
All Rights Reserved except where expressly stated otherwise.

You may read, study, run, and privately modify the Official Content Generator for
private, non-distributive use, including to build the game locally and to create original
user-generated content.

You may not redistribute the Official Content Generator, modified versions of it, or
substantial portions of it without explicit permission.

## Compilation and Distribution

You may clone, build, and run the project from source for private, non-distributive use.

You may not redistribute compiled builds, packages, archives, installers, or other
copies of the game that include Official Game Content or the Official Content Generator
except with explicit permission.

Engine Code may be modified and redistributed under the terms of the MPL 2.0,
provided that such redistribution does not include Official Game Content or the
Official Content Generator except as otherwise permitted.

## Game Format Documentation

Specifications for game content formats are provided under `docs/data/`
for reference and modding purposes, and are licensed under the Mozilla Public
License 2.0.

This grants permission to use documentation, schemas, and minimal examples
describing the game's public data formats to create compatible user-generated
content and modding tools, among other permissions granted under the MPL 2.0.

## Third-Party Content

Some third-party content is included under separate licenses. See
[LICENSE_THIRD_PARTY.md](LICENSE_THIRD_PARTY.md).

Please note that any third-party content included in the game is subject to the terms of its
respective licenses. Users must comply with the relevant third-party licenses when using,
redistributing, or modifying such content.

## User-Generated Content and Mods

Users may create, share, and distribute original content for the game, subject to the
modding terms below.

### Modding Terms

You may create, use, modify, and distribute your own independent datagen tools,
scripts, libraries, templates, and generators for creating User Generated Content,
including tools that produce files compatible with the game's supported data
formats.

You are granted a limited license to create derivative works based on Official Game Content
(such as modifying existing sprites, tilesets, or configuration files) specifically for
use as User Generated Content.

When distributing User Generated Content that contains or is derived from
Official Game Content, you must adhere to the following requirements:
- **Base Game Dependency**: The content must be designed to run strictly as a modification,
    add-on, or extension to the game, requiring the presence of the Official Game Content to function.
- **No Unmodified Redistribution**: You may only distribute the specific game files you have
    created or modified. You may not bundle or distribute unmodified Official Game Content
    alongside your mod.
- **No Substitution**: Your derivative works may not be distributed in a standalone
    capacity (e.g., repackaged as a separate standalone game) or in any manner that
    serves as a substitute for purchasing the original game.

User Generated Content, including works derivative of Official Game Content, may be distributed
for both commercial and non-commercial purposes, provided it complies with these terms and
does not include infringing third-party materials.

You may not use or distribute the Official Content Generator, or modified versions of
it, to recreate, extract, clone, or distribute Official Game Content.

Users retain ownership of their original additions in User Generated Content,
subject to any underlying rights in the Official Game Content or third-party materials.

User Generated Content may identify itself as compatible with the game, but may
not imply endorsement, sponsorship, or official status unless explicitly
authorized.

## Warranty Disclaimer

The software is provided "as is", without warranty of any kind, express or implied, including but not limited to the
warranties of merchantability, fitness for a particular purpose, and noninfringement. In no event shall the authors or
copyright holders be liable for any claim, damages, or other liability, whether in an action of contract, tort, or
otherwise, arising from, out of, or in connection with the software or the use or other dealings in the software.