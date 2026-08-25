use crate::obstacle;
use std::time::Duration;

pub const WIDTH: usize = 50;
pub const HEIGHT: usize = 20;

pub type GameGrid = [[char; WIDTH]; HEIGHT];

pub const TICKS_MS: u64 = 15; // ~66.67 FPS
pub const TICK_RATE: Duration = Duration::from_millis(TICKS_MS);

pub const BG_ICON: char = '.';
pub const PLAYER_ICON: char = '@';
pub const OBS_ICON: char = '■';

pub const DASH_LENGTH: usize = 9;
// const DASH_ICONS: [char; 4] = ['—', '–', '‑', '·'];
pub const DASH_ICONS: [char; 4] = ['█', '▓', '▒', '░'];

pub const DASH_EFFECT_CHANGE: Duration = Duration::from_millis(100);

// Obstacle spawning
pub const SPAWN_AFTER_MS: Duration = Duration::from_millis(300);
pub const INIT_MOVE_AFTER: Duration = Duration::from_millis(300);
pub const INCREASE_SPEED_BY: Duration = Duration::from_millis(10);
pub const INIT_SPEEDUP_AFTER: Duration = Duration::from_millis(1000);
pub const SLOW_SPEEDUP: Duration = Duration::from_millis(100);
pub const MIN_MOVE_AFTER: Duration = Duration::from_millis(TICKS_MS + 2);

pub const SPAWN_SHAPES: &[(obstacle::Shape, u32)] = &[
    (obstacle::Shape::Unit, 45),
    (obstacle::Shape::Triangle, 10),
    (
        obstacle::Shape::Brick {
            width: 5,
            height: 2,
        },
        25,
    ),
    (
        obstacle::Shape::Brick {
            width: 2,
            height: 6,
        },
        10,
    ),
    (
        obstacle::Shape::Brick {
            width: 10,
            height: 3,
        },
        10,
    ),
];
