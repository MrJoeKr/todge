use crate::obstacle;
use std::time::Duration;

pub mod keybinds;

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

pub const ULT_DURATION: Duration = Duration::from_millis(5000);
pub const BARRIER_ICON: char = '=';
/// Rows at or below this are swept when the ultimate fires.
pub const ULT_DESTROY_FROM_ROW: usize = HEIGHT - HEIGHT / 3;
pub const ULT_BAR_WIDTH: usize = 20;

// Obstacle spawning
pub const SPAWN_AFTER_MS: Duration = Duration::from_millis(200);
pub const INIT_SPEED: f64 = 5.0;
pub const INCREASE_SPEED_BY: f64 = 1.0;
pub const INIT_SPEEDUP_AFTER: Duration = Duration::from_millis(1000);
pub const SLOW_SPEEDUP: Duration = Duration::from_millis(100);
pub const MAX_SPEED: f64 = 64.67;

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

pub const OBSTACLE_TRAJECTORIES: &[(obstacle::Trajectory, u32)] = &[
    (obstacle::Trajectory::Straight, 80),
    (
        obstacle::Trajectory::Sine {
            amp: 3.0,
            freq: 0.5,
        },
        20,
    ),
];
