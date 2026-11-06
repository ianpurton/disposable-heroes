# CF1 import contract

Reference: OpenFodder/openfodder commit `0df22170a1c125be31a8aaef97fb190793c29d1b`, `Source/Map/Original.cpp`, `Source/Map/MapRuntime.cpp`, `Source/Tiles.hpp`, `Source/Structures/Barracks.cpp`. The Rust parser is independent of the C++ runtime.

- MAP: minimum 96-byte header. Base BLK filename starts at 0, subset at 16. Width and height are big-endian u16 at 84 and 86. Tile words begin at 96, row-major, big-endian. Extra file padding is allowed (Plus maps use it). Raw tile flags are retained; lookup uses `word & 511`.
- SPT: ten bytes per record, five big-endian words: direction, reserved, signed X, signed Y, sprite ID. Stored X is converted to simulation X by adding 16 exactly once. Negative stored coordinates are valid.
- HIT: signed big-endian i16 per tile. Low nibble is terrain; a negative value indicates a mixed tile. BHT supplies eight rows of eight bits, each cell covering 2×2 source pixels. A set bit selects the high terrain nibble. Bit 7 is leftmost.
- SWP: signed big-endian i16 tile replacement for destruction; negative means no substitution.
- Base resource tables start at index 0; subset tables start at 240. Resources are resolved from the actual header names, including `junsub1`.
- Sprite 4 is a reserved/null record. It is preserved but does not produce an object.
- Jungle barracks patterns group tiles into one structure associated with a door sprite. Unknown/incomplete patterns retain the source terrain. Visual anchors for additional structure families require further reference validation.

Tests use real map pairs, including 19×15 AmigaTheOne mapm1, 20×15 Segras First (stored X −2), sub1 Plus maps, padded MAP files and non-jungle parsing. Custom `alessandro` references desert metadata not present in the selected jungle fixtures; byte parsing succeeds, while missing terrain resources are a load error.
