# dmg-acid2

A PPU test ROM written by Matt Currie, used by `tests/dmg_acid2.rs`. It draws a face using
many of the PPU's rendering rules at once. Any rule that is wrong breaks part of the face.

- Source: <https://github.com/mattcurrie/dmg-acid2>
- `dmg-acid2.gb` is from the v1.0 release, and `reference-dmg.png` is `img/reference-dmg.png`
  from the repository. Both are unmodified.
- MIT licensed; see `LICENSE`.

The test compares the screen with `reference-dmg.png` shade by shade, so it doesn't depend
on the display palette. If it fails, it saves what the emulator drew as
`target/tmp/dmg-acid2-actual.png`.

The repository's README explains which part of the face checks which rule.
