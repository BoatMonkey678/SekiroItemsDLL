# Sekiro Item Give DLL
A `.dll` mod for Sekiro: Shadows Die Twice that allows to easily give items in-game.

## Installation
1. Choose version from the **Releases** tab.
For general usage, choose `sekiro_items.dll`. This one has a dropdown list for selecting items and is overall much easier to use.
The second file, `sekiro_items_dev.dll` doesn't have a dropdown list, but instead allows you to grant items by ID. This is useful for mod developers when testing something.

2. Use your favorite method for loading `.dll` mods. ME3 is the easiest, but can be unstable.

## Usage
When launching the game, a small overlay will appear.

In this overlay you can: enter the item and count to grant, and then grant them.

The overlay can be toggled on and off by pressing `Insert`

## Issues
There is currently an issue where two mouses appear when in-game. One of them works in-game and the other works on the overlay. These mouses unfortunately have desynced positions.

Another issue is with outfits. If you grant them early (like when arriving at Dilapidated Temple) and switch to them, then the outfit **Original Memory: Wolf** breaks. This is a quirk of how the outfits are treated by the game, not a problem with the tool itself. To fix this, grant yourself the item called **Original Memory: Wolf**.

If you spot any issues with items being mislabeled or not being granted, notify me through the **Issues** tab. I had to fill them by hand and might have missed something despite my best efforts.

## Credits
The following projects were used for this tool:
- [FromSoftware-rs](https://github.com/vswarte/fromsoftware-rs) - by vswarte for general bindings
- [FromSoftware-rs Fork](https://github.com/fswap/fromsoftware-rs) - by fswap - for dedicated Sekiro bindings
- [hudhook](https://github.com/veeenu/hudhook) - by veeenu
- [ilhook](https://github.com/regomne/ilhook-rs) - by regomne
- [serde](https://github.com/serde-rs/serde) - by dtolnay
- [anyhow](https://github.com/dtolnay/anyhow) - by dtolnay

## Buidling
Have Rust installed to build this project. Then run: `cargo build  --release` to build the `.dll` files.