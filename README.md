# Sekiro Item Give DLL
A `.dll` mod for Sekiro: Shadows Die Twice that allows to easily give items in-game.

## Installation
Use your favorite method for chainloading `.dll` mods.
For `.me3` profile add `path = "Path/To/Your/sekiro_items.dll"` to section `[[native]]`.

## Usage
When in-game an overlay menu appears where you can select (or type in by name) the item and quantity and then press a button to grant the item.
The menu can only be selected when the game is paused, since otherwise a mouse doesn't appear.
Most equippable/consumable items are there, all Combat Arts and Memories of Another (skins).

## Issues
There is currently an issue where two mouses appear when in-game. One of them works in-game and the other works on the overlay. These mouses unfortunately have desynced positions.

Another issue is with outfits. If you grant them early (like when arriving at Dilapidated Temple) and switch to them, then the outfit "Original Memory: Wolf" breaks. This is a quirk of how the outfits are treated by the game (they are basically treated like Dark Souls armor pieces), not a problem with the tool itself. To fix this, grant yourself the item called "Original Memory: Wolf".

If you spot any issues with items being mislabeled or not being granted, notify me through the "Issues" tab. I had to fill them by hand and might have missed something despite my best efforts.

## Credits
The following projects were used for this tool:
- [FromSoftware-rs](https://github.com/vswarte/fromsoftware-rs) - by vswarte for general bindings
- [FromSoftware-rs Fork](https://github.com/fswap/fromsoftware-rs) - fork by fswap - for dedicated Sekiro bindings
- [hudhook](https://github.com/veeenu/hudhook) - by veeenu
- [ilhook](https://github.com/regomne/ilhook-rs) - by regomne
- [serde](https://github.com/serde-rs/serde) - by dtolnay
- [anyhow](https://github.com/dtolnay/anyhow) - by dtolnay

## Buidling
I used Rust version: `rustc 1.96.0-nightly` to build this project. With this run:
`cargo build -p sekiro-items --release`
to build the `.dll` file.
