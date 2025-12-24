# S-Force

A top-down space fighter prototype built with Bevy. Development progresses in explicit phases per the project brief.

## Phase 1 Status
- Core loop online: Title → Gameplay → Game Over → restart via Title screen.
- Player ship spawns with WASD/Arrow movement bounded to the screen.
- Hold Space/Enter to fire forward shots with cooldown; bullets despawn off-screen.
- Basic enemy wave (straight-flying grunts) spawns in lanes; bullets destroy them for score.
- Simple HUD shows score and remaining lives; colliding with an enemy ends the run.

## Run Instructions
```sh
cargo run
```

## Manual Test Checklist
1. Run `cargo run`; confirm the 1280×720 window opens on the title screen.
2. Press `Space` or `Enter` to start the run and observe the HUD in the top-left.
3. Use WASD / Arrow keys to move; verify the ship stays within the play area.
4. Hold `Space` to fire. Hit an enemy and confirm the score text increments by 100.
5. Run into an enemy; lives decrement and a Game Over overlay appears.
6. Press `Enter`/`Space` on the Game Over screen to return to the Title screen, then restart another run.
