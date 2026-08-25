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

impl Shape {
    fn offsets(self) -> Vec<(i32, i32)> {
        match self {
            Shape::Unit => vec![(0, 0)],
            Shape::Triangle => vec![(0, 0), (-1, 1), (1, 1)],
            Shape::Brick { width, height } => {
                let (w, h) = (width as i32, height as i32);
                (0..w)
                    .flat_map(move |dx| (0..h).map(move |dy| (dx - w / 2, dy - h / 2)))
                    .collect()
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Obstacle {
    /// Anchor in continuous grid space.
    pub spawn_x: f64,
    pub spawn_y: f64,
    /// Cells/second moving downward.
    pub speed: f64,
    pub shape: Shape,
    pub trajectory: Trajectory,
    pub age: Duration,
}

impl Obstacle {
    fn x(&self) -> f64 {
        self.spawn_x
            + match self.trajectory {
                Trajectory::Straight => 0.0,
                Trajectory::Sine { amp, freq } => {
                    amp * (freq * self.age.as_secs_f64() * std::f64::consts::TAU).sin()
                }
            }
    }

    fn y(&self) -> f64 {
        self.spawn_y + self.speed * self.age.as_secs_f64()
    }

    pub fn update(&mut self, dt: Duration) {
        self.age += dt;
    }

    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let (ax, ay) = (self.x().round() as i32, self.y().round() as i32);
        self.shape
            .offsets()
            .into_iter()
            .map(move |(dx, dy)| (ax + dx, ay + dy))
    }

    pub fn render(&self, game_grid: &mut GameGrid) {
        for (x, y) in self.cells() {
            if let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y))
                && y < HEIGHT
                && x < WIDTH
            {
                game_grid[y][x] = OBS_ICON
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum Trajectory {
    #[default]
    Straight,
    Sine {
        amp: f64,
        freq: f64,
    },
}
