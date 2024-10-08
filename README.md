# YAEmulator-Rust

## GB Structure
![GB Structure](./GB_Schema.png)

## GB Memory Map
![Memory Map](./Memory_Map.png)

## Status
CPU and PPU are doing their work as expected. Need to add real user inputs and then audio. Note that this not work directly with cmd pixel but with special characters (using low font size as 4px)
![Super Mario first frame](./Super_Mario_frame_cmd.png)

## References
[GBDev.io](https://gbdev.io/pandocs/About.html) - Main general references for GB/GBC hardware

[Game Boy Official Programming Manual](https://ia903208.us.archive.org/9/items/GameBoyProgManVer1.1/GameBoyProgManVer1.1.pdf) - Official Nintendo documentation about GB (v 1.1). Thank you archive.org

[Copetti](https://www.copetti.org/writings/consoles/game-boy/) - Lite reading about GB/GBC hardware functionalities

[Rylev DMG-01](https://rylev.github.io/DMG-01/public/book/introduction.html) - Secondary reference about how a Game Boy works