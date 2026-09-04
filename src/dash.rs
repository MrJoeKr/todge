use crate::config::{DASH_EFFECT_CHANGE, DASH_ICONS, DASH_LENGTH, GameGrid};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct DashEffectCollection {
    pub container: HashMap<(usize, usize), DashEffect>,
}

impl DashEffectCollection {
    pub fn render(&self, game_grid: &mut GameGrid) {
        for ((x, y), de) in &self.container {
            game_grid[*y][*x] = DASH_ICONS[de.state_idx];
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.container.retain(|_k, de| {
            de.time_since_change += dt;
            if de.time_since_change >= DASH_EFFECT_CHANGE {
                de.state_idx += 1;
                de.time_since_change -= DASH_EFFECT_CHANGE;
            }
            de.state_idx < DASH_ICONS.len()
        });
    }

    /// Same fade for all cells, starting from the newest dash effect icon
    pub fn add_burst(&mut self, cells: impl IntoIterator<Item = (usize, usize)>) {
        for cell in cells {
            self.container.insert(cell, DashEffect::default());
        }
    }

    pub fn add(&mut self, cells: Vec<(usize, usize)>) {
        assert_eq!(cells.len(), DASH_LENGTH + 1);

        let give_times = DASH_LENGTH / DASH_ICONS.len();

        if give_times == 0 {
            for cell in cells {
                self.container.insert(cell, DashEffect::default());
            }
            return;
        }

        let mut state_idx = 0;
        let mut cell_idx = 0;
        while state_idx < DASH_ICONS.len() {
            for _ in 0..give_times {
                self.container.insert(
                    cells[cell_idx],
                    DashEffect {
                        state_idx,
                        ..Default::default()
                    },
                );
                cell_idx += 1;
            }
            state_idx += 1;
        }

        let rem = DASH_LENGTH % DASH_ICONS.len();
        // assert_eq!(idx, 0);
        for _ in 0..rem {
            self.container.insert(
                cells[cell_idx],
                DashEffect {
                    state_idx: state_idx - 1,
                    ..Default::default()
                },
            );
            cell_idx += 1;
        }
    }
}

#[derive(Debug, Default)]
pub struct DashEffect {
    state_idx: usize,
    time_since_change: Duration,
}
