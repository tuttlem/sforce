# S-Force

A top-down space fighter built with Bevy. Development progresses in explicit phases per the project brief.

## Phase 3 Status
- Parallax starfield and procedural audio tones provide continuous ambience.
- Title screen exposes difficulty and per-channel volume controls (Tab to cycle difficulty, `-`/`+` for music, `[`/`]` for SFX).
- Pause/resume supported at any time (`P`/`Esc`), alongside a toggleable debug overlay (`F3`) that reports FPS, entity count, and wave index.
- Boss encounter unlocks after surviving sufficient waves; it features three attack phases, a dedicated HUD health bar, and defeating it returns the loop to the title screen.
- Background music loops automatically; shoot, hit, explosion, pickup, and UI actions trigger synthesized SFX.
- Ships draw from the tinyShip sprite sheets (tinyShip3 for player, tinyShip1/5/7/10/13 for enemies, tinyShip20 for bosses), and bullets/power-ups/explosions use animated frames from `explosions.png` for a unified pixel aesthetic.
*** End Patch
