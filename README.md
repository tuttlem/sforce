# S-Force

A top-down space fighter prototype built with Bevy. Development progresses in explicit phases per the project brief.

## Phase 2 Status
- Five enemy archetypes now rotate through scripted waves with deterministic spawn timing.
- Enemies fire different projectile styles (straight, targeted, spread) and the player has lives with invulnerability frames and flashing feedback.
- Player weapons support tiered patterns (single, twin, spread, laser) with faster fire rates unlocked via pickups.
- Three power-ups (Spread, Rapid, Shield) drop on a timer and can be collected mid-fight.
- HUD/Game flow preserved: Title → Gameplay → Game Over → restart, with lives/score updates.

## Run Instructions
```sh
cargo run
```

## Manual Test Checklist
1. Run `cargo run` and confirm the title screen appears; start with `Space`/`Enter`.
2. Observe HUD score/lives in the gameplay state.
3. Move with WASD / Arrow keys; verify the ship stays within bounds.
4. Hold `Space` or left-click to fire; confirm weapon upgrades progress as power-ups are collected.
5. Destroy multiple waves and observe varied enemy types (straight, sine, zigzag, tanks, chasers) and their projectile patterns.
6. Take a hit from an enemy or projectile; lives decrement and the ship flashes while invulnerable.
7. Collect each power-up (green Spread, blue Rapid, gold Shield) and see the corresponding weapon/shield effect.
8. Lose all lives to reach Game Over, then press `Space`/`Enter` to return to the title screen and restart.
