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
    pub cells: Vec<(usize, usize)>,
    pub move_after: Duration,
    pub time_since_move: Duration,
}

impl Obstacle {
    pub fn new(x: usize, y: usize, move_after: Duration, shape: Shape) -> Self {
        Obstacle {
            move_after,
            cells: match shape {
                Shape::Unit => vec![(x, y)],
                Shape::Triangle => vec![(x, y), (x - 1, y + 1), (x + 1, y + 1)],
                Shape::Brick { width, height } => {
                    // TODO: For now, it's top-left corner (would be nice if the brick came from top
                    // out of bounds later
                    let mut out = Vec::new();
                    for dx in 0..width {
                        for dy in 0..height {
                            out.push((x + dx, y + dy));
                        }
                    }
                    out
                }
            },
            ..Default::default()
        }
    }

    pub fn move_obstacle(&mut self, dt: Duration) {
        self.time_since_move += dt;
        if self.time_since_move >= self.move_after {
            self.time_since_move -= self.move_after;
            for (_x, y) in &mut self.cells {
                *y += 1;
            }
        }
    }

    pub fn render(&self, game_grid: &mut GameGrid) {
        for (x, y) in &self.cells {
            if *y < HEIGHT && *x < WIDTH {
                game_grid[*y][*x] = OBS_ICON
            }
        }
    }
}
