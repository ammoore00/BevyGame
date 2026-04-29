## Definitions

"Engine Code" and "Game Engine" refer to the code used to load and run content for the game.
This is the code under `src/`, as well as sub-crates excluding the code under `datagen/`.

"Official Game Content" refers to all game assets, data, and content provided by the
project, including images, audio, fonts, maps, levels, dialogue, characters,
configuration files, generated data files, source asset files, and other non-engine
content, except where otherwise stated.

The "Official Content Generator" refers to the code used to generate the game's
official game content, located under `datagen/`.

"User Generated Content" refers to original mods, game content, and content
generation tools created by end users. User Generated Content does not include
Official Game Content, the Official Content Generator, modified versions of the
Official Content Generator, or substantial portions of any of them.

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

The following assets required for basic engine functionality are exempt
and are provided under the terms of the MPL 2.0:
- `assets/base/images/ui/bars.png`
- `assets/base/images/ui/buttons.png`

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

User Generated Content may be based on the game’s data formats, public APIs, and
workflows, but may not directly use, extract, or redistribute Official Game Content
or the Official Content Generator. Any derivative works must rely solely on the
Engine Code and user-provided assets.

User Generated Content may not include or redistribute Official Game Content or
third-party content included with the game, except as allowed by the applicable
license or by explicit permission.

You may not use or distribute the Official Content Generator, or modified versions of
it, to recreate, extract, clone, or distribute Official Game Content.

Users retain ownership of their original User Generated Content, subject to
any third-party materials they include.

User Generated Content may be distributed for both commercial and non-commercial
purposes, provided it complies with these terms and does not include Official Game
Content, the Official Content Generator, or infringing third-party materials.

User Generated Content may identify itself as compatible with the game, but may
not imply endorsement, sponsorship, or official status unless explicitly
authorized.

## Warranty Disclaimer

The software is provided "as is", without warranty of any kind, express or implied, including but not limited to the
warranties of merchantability, fitness for a particular purpose and noninfringement. In no event shall the authors or
copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or
otherwise, arising from, out of or in connection with the software or the use or other dealings in the software.