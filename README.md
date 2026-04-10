# Sekiro Item Give DLL
A `.dll` mod for Sekiro: Shadows Die Twice that allows for giving items in-game easily.

## Installation
1. Pick a version:
   - `sekiro_items.dll` - for general usage
   - `sekiro_items_dev.dll` - for testing mods, as it allows to grant items by ID

2. Use your favorite method for loading `.dll` mods. ME3 is the easiest, but can be unstable.

## Usage
When launching the game, a small overlay will appear.

In this overlay you can: enter the item and count and grant them.

The overlay can be toggled on and off by pressing `Insert`

## Issues
If you run into any problems with items being mislabeled or not being granted, please open an **Issue**. I had to fill them by hand and might have missed something despite my best efforts.

There is currently an issue where two mouses appear when in-game. One of them works in-game and the other works on the overlay. These mouses unfortunately have desynced positions.

If you grant Outfits early and switch to them, then **Original Memory: Wolf** breaks. This is a quirk of how the outfits are treated by the game, not a problem with the tool itself. To fix this, grant yourself the item called **Original Memory: Wolf**.


## Credits
The following projects were used while developing this tool:
- [FromSoftware-rs](https://github.com/vswarte/fromsoftware-rs) - by vswarte - for general bindings
- [FromSoftware-rs Fork](https://github.com/fswap/fromsoftware-rs) - by fswap - for dedicated Sekiro bindings
- [hudhook](https://github.com/veeenu/hudhook) - by veeenu
- [ilhook](https://github.com/regomne/ilhook-rs) - by regomne
- [serde](https://github.com/serde-rs/serde) - by dtolnay
- [anyhow](https://github.com/dtolnay/anyhow) - by dtolnay

## Buidling
Have Rust installed to build this project. Then run: `cargo build  --release` to build the `.dll` files.