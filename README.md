# S-Force

A top-down space fighter built with Bevy. Development progresses in explicit phases per the project brief.

## Phase 3 Status
- Parallax starfield and procedural audio tones provide continuous ambience.
- Title screen exposes difficulty and per-channel volume controls (Tab to cycle difficulty, `-`/`+` for music, `[`/`]` for SFX).
- Pause/resume supported at any time (`P`/`Esc`), alongside a toggleable debug overlay (`F3`) that reports FPS, entity count, and wave index.
- Boss encounter unlocks after surviving sufficient waves; it features three attack phases, a dedicated HUD health bar, and defeating it returns the loop to the title screen.
- Background music loops automatically; shoot, hit, explosion, pickup, and UI actions trigger synthesized SFX.
- Ships and enemies now use atlas-driven sprites from the `tinyShip*` sheets (player uses tinyShip3, enemies mix several sizes, and the boss uses tinyShip20) with animated thrusters and appropriately scaled collision bounds.

## Run Instructions
```sh
cargo run
```

## Manual Test Checklist
1. Run `cargo run` and confirm the 1280×720 window opens on the title screen.
2. Use `Tab`, `-`/`+`, and `[`/`]` to adjust difficulty, music, and SFX levels; verify the text reflects changes.
3. Press `Space`/`Enter` to start the run. Observe the parallax background and HUD (score, lives, boss bar hidden).
4. Move with WASD / Arrow keys; hold `Space` or left-click to fire. Confirm SFX volume tracks the current slider.
5. Destroy enemies and collect the three power-ups (Spread, Rapid, Shield). Check that each alters fire patterns or shields accordingly.
6. Press `P` (or `Esc`) during gameplay to pause, then resume with the same key. Toggle the debug overlay with `F3` and confirm FPS/entity/wave numbers update.
7. After surviving enough waves, verify the boss spawns, its health bar becomes visible, and it executes multiple patterns before defeat returns the game to the title screen.
8. Lose all lives to enter the Game Over overlay, then press `Enter`/`Space` to return to the title screen.
