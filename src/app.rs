use crate::config::*;
use crate::dash::DashEffectCollection;
use crate::obstacle::Obstacle;
use crate::player::{Player, PlayerDirection, PlayerState};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::{
    distr::{self, Distribution},
    seq::IndexedRandom,
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::io;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct App {
    game_grid: GameGrid,
    exit: bool,
    game_over: bool,
    wants_restart: bool,

    player: Player,

    // Obstacles
    obstacles: HashMap<u64, Obstacle>, // { id: Obs }
    obs_id: u64,
    obs_speed: f64,
    speedup_after: Duration,
    time_after_spawn: Duration,
    time_after_speedup: Duration,

    dash_collection: DashEffectCollection,

    elapsed_time: Duration,

    rng: rand::prelude::ThreadRng,
    distr: distr::Uniform<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            game_grid: [[BG_ICON; WIDTH]; HEIGHT],
            exit: false,
            game_over: false,
            wants_restart: false,

            player: Player {
                x: WIDTH / 2,
                y: HEIGHT - 1,
                ..Default::default()
            },

            obstacles: HashMap::new(),
            obs_id: 0,
            obs_speed: INIT_SPEED,
            speedup_after: INIT_SPEEDUP_AFTER,
            time_after_spawn: Duration::ZERO,
            time_after_speedup: Duration::ZERO,

            dash_collection: DashEffectCollection::default(),

            elapsed_time: Duration::ZERO,

            rng: rand::rng(),
            distr: distr::Uniform::new(0, WIDTH).unwrap(),
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.init_game();
        let mut last_update = Instant::now();

        while !self.exit {
            // Poll for input (non‑blocking)
            if event::poll(Duration::ZERO)?
                && let Event::Key(key_event) = event::read()?
                && (key_event.kind == KeyEventKind::Press || key_event.kind == KeyEventKind::Repeat)
            {
                self.handle_key_event(key_event);
            }

            if !self.game_over && last_update.elapsed() >= TICK_RATE {
                self.update_game(last_update.elapsed());
                last_update = Instant::now();
            }

            if self.game_over && self.wants_restart {
                self.restart_game();
                last_update = Instant::now();
            }

            terminal.draw(|frame| self.draw(frame))?;

            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    fn init_game(&mut self) {
        self.game_grid[self.player.y][self.player.x] = PLAYER_ICON;
    }

    fn restart_game(&mut self) {
        *self = Self::default();
    }

    fn add_obstacle(&mut self, obs: Obstacle) {
        self.obstacles.insert(self.obs_id, obs);
        self.obs_id += 1;
    }

    fn try_spawn_obstacle(&mut self, dt: Duration) {
        self.time_after_spawn += dt;
        self.time_after_speedup += dt;
        if self.time_after_spawn >= SPAWN_AFTER_MS {
            self.spawn_obstacle();
        }

        if self.time_after_speedup >= self.speedup_after {
            self.time_after_speedup -= self.speedup_after;

            self.obs_speed = MAX_SPEED.min(self.obs_speed + INCREASE_SPEED_BY);
            self.speedup_after += SLOW_SPEEDUP;
        }
    }

    fn spawn_obstacle(&mut self) {
        self.time_after_spawn -= SPAWN_AFTER_MS;
        let x = self.distr.sample(&mut self.rng);
        let chosen_shape = SPAWN_SHAPES
            .choose_weighted(&mut self.rng, |item| item.1)
            .unwrap()
            .0;

        let chosen_trajectory = OBSTACLE_TRAJECTORIES
            .choose_weighted(&mut self.rng, |item| item.1)
            .unwrap()
            .0;

        let dya = -1.0;
        self.add_obstacle(Obstacle::new(
            (x as f64, dya),
            self.obs_speed,
            chosen_shape,
            chosen_trajectory,
        ));
    }

    fn update_obstacles(&mut self, dt: Duration) {
        // Clean obstacles that are OBS
        self.obstacles.retain(|_, obs| {
            obs.update(dt);
            obs.cells().any(|(_, y)| y < HEIGHT as i32)
        });
    }

    fn move_player(&mut self) {
        match self.player.move_state {
            PlayerState::Moving(ref p_dir) => match p_dir {
                PlayerDirection::Left => self.go_left(),
                PlayerDirection::Right => self.go_right(),
            },
            PlayerState::Dashing(ref p_dir) => match p_dir {
                PlayerDirection::Left => self.dash_left(),
                PlayerDirection::Right => self.dash_right(),
            },
            PlayerState::Idle => (),
        }
    }

    fn check_collision(&self) -> bool {
        match self.player.move_state {
            PlayerState::Moving(_) | PlayerState::Idle => {
                self.check_collision_helper(&[(self.player.x, self.player.y)])
            }
            PlayerState::Dashing(ref p_dir) => {
                // `move_player()` already moved the player to the dash destination.
                // Use the opposite direction to check collision.
                // Ty `DeepSeek V4 Flash 0731` for finding this bug.
                let direction = p_dir.opposite();
                let dash_cells = self.construct_dash_cells(&direction);
                self.check_collision_helper(&dash_cells)
            }
        }
    }

    /// Return positions in chronological order of dashing
    fn construct_dash_cells(&self, p_dir: &PlayerDirection) -> Vec<(usize, usize)> {
        let mut out = vec![(self.player.x, self.player.y)];
        let mut px = self.player.x;
        let py = self.player.y;
        match p_dir {
            PlayerDirection::Left => {
                for _ in 0..DASH_LENGTH {
                    px = (px + WIDTH - 1) % WIDTH;
                    out.push((px, py));
                }
            }
            PlayerDirection::Right => {
                for _ in 0..DASH_LENGTH {
                    px = (px + 1) % WIDTH;
                    out.push((px, py));
                }
            }
        }

        out
    }

    fn check_collision_helper(&self, targets: &[(usize, usize)]) -> bool {
        targets.iter().any(|&(tx, ty)| {
            self.obstacles
                .values()
                .any(|obs| obs.cells().any(|cell| cell == (tx as i32, ty as i32)))
        })
    }

    fn drop_colliding_dashes_helper(&mut self, targets: &[(usize, usize)]) {
        self.dash_collection
            .container
            .retain(|pos, _de| !targets.contains(pos));
    }

    fn drop_colliding_dashes(&mut self) {
        // Player + obstacles
        let mut targets = vec![self.player.position()];

        let obstacles = self.obstacles.values().flat_map(|obs| {
            obs.cells()
                .filter(|&(x, y)| 0 <= x && x <= WIDTH as i32 && 0 <= y && y <= HEIGHT as i32)
                .map(|(x, y)| (x as usize, y as usize))
        });
        targets.extend(obstacles);

        self.drop_colliding_dashes_helper(&targets);
    }

    fn update_game(&mut self, dt: Duration) {
        self.update_obstacles(dt);
        self.move_player();
        self.game_over = self.check_collision();

        self.try_spawn_obstacle(dt);

        self.dash_collection.update(dt);
        self.drop_colliding_dashes();

        self.update_grid();

        self.elapsed_time += dt;
        self.player.move_state = PlayerState::Idle;
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') => {
                if self.game_over {
                    self.wants_restart = true;
                }
            }
            KeyCode::Char('h') => {
                self.player.move_state = PlayerState::Moving(PlayerDirection::Left)
            }
            KeyCode::Char('l') => {
                self.player.move_state = PlayerState::Moving(PlayerDirection::Right)
            }
            KeyCode::Char('H') => {
                self.player.move_state = PlayerState::Dashing(PlayerDirection::Left)
            }
            KeyCode::Char('L') => {
                self.player.move_state = PlayerState::Dashing(PlayerDirection::Right)
            }
            _ => (),
        }
    }

    fn get_final_score(&self) -> u128 {
        self.elapsed_time.as_millis() / 10
    }

    fn go_left(&mut self) {
        self.player.x = (self.player.x + WIDTH - 1) % WIDTH;
    }

    fn go_right(&mut self) {
        self.player.x = (self.player.x + 1) % WIDTH;
    }

    fn dash_left(&mut self) {
        self.player.x = (self.player.x + WIDTH - DASH_LENGTH) % WIDTH;

        let cells = self.construct_dash_cells(&PlayerDirection::Right);
        // tracing::debug!(c = ?cells, "Dash Left");
        self.dash_collection.add(cells.to_vec());
    }

    fn dash_right(&mut self) {
        self.player.x = (self.player.x + DASH_LENGTH) % WIDTH;

        let cells = self.construct_dash_cells(&PlayerDirection::Left);
        // tracing::debug!(c = ?cells, "Dash Right");
        self.dash_collection.add(cells.to_vec());
    }

    fn clear_board(&mut self) {
        for row in &mut self.game_grid {
            row.fill(BG_ICON);
        }
    }

    fn update_grid(&mut self) {
        self.clear_board();

        self.game_grid[self.player.y][self.player.x] = PLAYER_ICON;

        self.dash_collection.render(&mut self.game_grid);

        for obs in self.obstacles.values() {
            obs.render(&mut self.game_grid);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" todge. ".bold());
        let instructions = Line::from(vec![
            " Left ".into(),
            "<h/H>".blue().bold(),
            " Right ".into(),
            "<l/L>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // 2D array -> vector of lines
        let mut lines: Vec<Line> = self
            .game_grid
            .iter()
            .map(|row| {
                Line::from(
                    row.iter()
                        .map(|&c| match c {
                            OBS_ICON => Span::from(c.to_string()).red(),
                            PLAYER_ICON => Span::from(c.to_string()).green(),
                            BG_ICON => Span::from(c.to_string()).blue(),
                            _ => Span::from(c.to_string()),
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        let game_over_text = vec![
            Line::from("Game Over!"),
            Line::from(vec![
                "Final Score: ".into(),
                self.get_final_score().to_string().yellow(),
            ]),
            Line::from("Restart? <r>".blue()),
        ];

        let game_on_text = vec![
            Line::from(""),
            Line::from(vec![
                "Score: ".into(),
                format!("{:>5}", self.get_final_score()).yellow(),
            ]),
            Line::from(vec![
                "Speed: ".into(),
                format!("{:>5} cells/s", self.obs_speed).blue(),
            ]),
        ];
        lines.extend(if !self.game_over {
            game_on_text
        } else {
            game_over_text
        });

        let game_text = Text::from(lines);

        Paragraph::new(game_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
