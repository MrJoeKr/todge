# TODO

* [ ] Find a good way to init obstacle instead of using `::new`

* [x] Refactor and organize to files
* [ ] Add various movement
* [ ] Different colors for different shapes
* [ ] Maybe special superpower to destroy obstacles for a short period of time
* [ ] Add special shape of a simple maze to go through

* [x] FIX: bug where dashing collides with longs gone obstacles

* [x] Fix handles keys events to just change bools (moved_left, ...)
* [x] Add player dashing

* [x] Add restart button after game over
* [x] Random shapes spawning
* [x] Test scenarios (`cargo run -- -s <name>`, see `src/scenario.rs`)

## Mechanics

* [ ] Make obstacles spawn out of upper bounds and then come down

* [ ] Cool mechanic idea:
    - Add the option that the player can move up on the grid.
    - When he is above ground, there's a progress bar for an ultimate ability. The closer the player is to the ceiling, the quicker the progress bar increases. (linearly: ultimate_bar = K * p_y)
    - Once the bar is 100%, the player can use a button to use it
    - Using it teleports them on the ground, giving invincibility

* [ ] Small chance of extra life powerup falling from the sky

## Study Rust Concepts

* [ ] Lifetimes
* [ ] opaque return types
* [x] Modules, crates, use
