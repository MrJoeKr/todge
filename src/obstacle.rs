use crate::config::{GameGrid, HEIGHT, OBS_ICON, WIDTH};
use std::time::Duration;

#[derive(Debug, Default, Copy, Clone)]
pub enum Shape {
    #[default]
    Unit,
    Triangle,
    Brick {
        width: usize,
        height: usize,
    },
}

#[derive(Debug, Default)]
pub struct Obstacle {
    /// Anchor in continuous grid space.
    pos: (f64, f64),
    spawn_x: f64,
    /// Integer offsets from the anchor.
    offsets: Vec<(i32, i32)>,
    /// Cells per second moving downward.
    speed: f64,
    trajectory: Trajectory,
    age: Duration,
}

impl Obstacle {
    pub fn new(anchor: (f64, f64), speed: f64, shape: Shape, trajectory: Trajectory) -> Self {
        let (x, y) = (anchor.0.round() as i32, anchor.1.round() as i32);
        Obstacle {
            pos: anchor,
            spawn_x: anchor.0,
            speed,
            offsets: match shape {
                Shape::Unit => vec![(x, y)],
                Shape::Triangle => vec![(x, y), (x - 1, y + 1), (x + 1, y + 1)],
                Shape::Brick { width, height } => {
                    let x0 = (anchor.0 - (width as f64) / 2.0).round() as i32;
                    let y0 = (anchor.1 - (height as f64) / 2.0).round() as i32;
                    let mut out = Vec::new();
                    for dx in 0..width as i32 {
                        for dy in 0..height as i32 {
                            out.push((x0 + dx, y0 + dy));
                        }
                    }
                    out
                }
            },
            trajectory,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.age += dt;
        self.pos.1 += self.speed * dt.as_secs_f64();
        self.pos.0 = self.spawn_x + match self.trajectory {
            Trajectory::Straight => 0.0,
            Trajectory::Sine { amp, freq } => {
                amp * (freq * self.age.as_secs_f64() * std::f64::consts::TAU).sin()
            }
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let (ax, ay) = (self.pos.0.round() as i32, self.pos.1.round() as i32);
        self.offsets.iter().map(move |&(dx, dy)| (ax + dx, ay + dy))
    }

    pub fn render(&self, game_grid: &mut GameGrid) {
        for (x, y) in self.cells() {
            if let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y))
                && y < HEIGHT && x < WIDTH {
                game_grid[y][x] = OBS_ICON
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Trajectory {
    #[default]
    Straight,
    Sine { amp: f64, freq: f64 },
}
