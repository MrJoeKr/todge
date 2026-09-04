use crate::config::*;
use crate::dash::DashEffectCollection;
use crate::obstacle::Obstacle;
use crate::player::{HDirection, Player, PlayerState, VDirection};
use crate::ultimate::Ultimate;

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
    ultimate: Ultimate,

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
            ultimate: Ultimate::default(),

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

        self.add_obstacle(Obstacle {
            spawn_x: x as f64,
            spawn_y: -1.0,
            speed: self.obs_speed,
            shape: chosen_shape,
            trajectory: chosen_trajectory,
            ..Default::default()
        });
    }

    fn update_obstacles(&mut self, dt: Duration) {
        // Clean obstacles that are OBS
        self.obstacles.retain(|_, obs| {
            obs.update(dt);
            obs.cells().any(|(_, y)| y < HEIGHT as i32)
        });
    }

    /// Vaporize every obstacle caught by `doomed`, leaving a fade effect.
    fn destroy_obstacles(&mut self, doomed: impl Fn(&Obstacle) -> bool) {
        let mut vaporize_cells = Vec::new();

        self.obstacles.retain(|_, obs| {
            if !doomed(obs) {
                return true;
            }

            vaporize_cells.extend(
                obs.cells()
                    .filter_map(|(x, y)| Some((usize::try_from(x).ok()?, usize::try_from(y).ok()?)))
                    .filter(|&(x, y)| x < WIDTH && y < HEIGHT),
            );
            false
        });

        self.dash_collection.add_burst(vaporize_cells);
    }

    /// Vaporize obstacles in the lower part of the board once when the ultimate fires.
    fn destroy_lower_obstacles(&mut self) {
        self.destroy_obstacles(|obs| obs.cells().any(|(_, y)| y >= ULT_DESTROY_FROM_ROW as i32));
    }

    /// Per tick while the ultimate runs: anything that would have hit the player.
    fn destroy_touching_obstacles(&mut self) {
        let (px, py) = self.player.position();
        // TODO(fix): dashing not considered
        self.destroy_obstacles(|obs| obs.cells().any(|cell| cell == (px as i32, py as i32)));
    }

    fn move_player(&mut self) {
        match self.player.move_state {
            PlayerState::Moving(ref h_dir) => match h_dir {
                HDirection::Left => self.go_left(),
                HDirection::Right => self.go_right(),
            },
            PlayerState::Dashing(ref h_dir) => match h_dir {
                HDirection::Left => self.dash_left(),
                HDirection::Right => self.dash_right(),
            },
            PlayerState::Flying(ref v_dir) => match v_dir {
                VDirection::Up => self.go_up(),
                VDirection::Down => self.go_down(),
            },
            PlayerState::Idle => (),
        }
    }

    fn check_collision(&self) -> bool {
        match self.player.move_state {
            PlayerState::Moving(_) | PlayerState::Flying(_) | PlayerState::Idle => {
                self.check_collision_helper(&[(self.player.x, self.player.y)])
            }
            PlayerState::Dashing(p_dir) => {
                // `move_player()` already moved the player to the dash destination.
                // Use the opposite direction to check collision.
                let direction = p_dir.opposite();
                let dash_cells = self.construct_dash_cells(direction);
                self.check_collision_helper(&dash_cells)
            }
        }
    }

    /// Return positions in chronological order of dashing
    fn construct_dash_cells(&self, p_dir: HDirection) -> Vec<(usize, usize)> {
        let mut out = vec![(self.player.x, self.player.y)];
        let mut px = self.player.x;
        let py = self.player.y;
        match p_dir {
            HDirection::Left => {
                for _ in 0..DASH_LENGTH {
                    px = (px + WIDTH - 1) % WIDTH;
                    out.push((px, py));
                }
            }
            HDirection::Right => {
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
        self.ultimate.update(dt, self.player.altitude());
        self.update_obstacles(dt);
        self.move_player();

        if self.ultimate.is_active() {
            self.destroy_touching_obstacles();
        } else {
            self.game_over = self.check_collision();
        }

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
            KeyCode::Char(keybinds::QUIT) => self.exit(),
            KeyCode::Char(keybinds::RESTART) => {
                if self.game_over {
                    self.wants_restart = true;
                }
            }
            KeyCode::Char(keybinds::LEFT) => {
                self.player.move_state = PlayerState::Moving(HDirection::Left)
            }
            KeyCode::Char(keybinds::RIGHT) => {
                self.player.move_state = PlayerState::Moving(HDirection::Right)
            }
            KeyCode::Char(keybinds::DASH_LEFT) => {
                self.player.move_state = PlayerState::Dashing(HDirection::Left)
            }
            KeyCode::Char(keybinds::DASH_RIGHT) => {
                self.player.move_state = PlayerState::Dashing(HDirection::Right)
            }
            KeyCode::Char(keybinds::UP) => {
                self.player.move_state = PlayerState::Flying(VDirection::Up)
            }
            KeyCode::Char(keybinds::DOWN) => {
                self.player.move_state = PlayerState::Flying(VDirection::Down)
            }
            KeyCode::Char(keybinds::ULTIMATE) => self.try_use_ultimate(),
            _ => (),
        }
    }

    fn try_use_ultimate(&mut self) {
        if self.ultimate.activate() {
            self.player.y = HEIGHT - 1; // slam to the ground
            self.destroy_lower_obstacles();
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

    fn go_up(&mut self) {
        if self.ultimate.is_active() {
            return; // the barrier keeps the player grounded
        }

        self.player.y = self.player.y.saturating_sub(1);
    }

    fn go_down(&mut self) {
        self.player.y = (self.player.y + 1).min(HEIGHT - 1);
    }

    fn dash_left(&mut self) {
        self.player.x = (self.player.x + WIDTH - DASH_LENGTH) % WIDTH;

        let cells = self.construct_dash_cells(HDirection::Right);
        // tracing::debug!(c = ?cells, "Dash Left");
        self.dash_collection.add(cells.to_vec());
    }

    fn dash_right(&mut self) {
        self.player.x = (self.player.x + DASH_LENGTH) % WIDTH;

        let cells = self.construct_dash_cells(HDirection::Left);
        // tracing::debug!(c = ?cells, "Dash Right");
        self.dash_collection.add(cells.to_vec());
    }

    fn update_grid(&mut self) {
        self.game_grid.iter_mut().for_each(|row| row.fill(BG_ICON));

        self.game_grid[self.player.y][self.player.x] = PLAYER_ICON;

        self.dash_collection.render(&mut self.game_grid);

        for obs in self.obstacles.values() {
            obs.render(&mut self.game_grid);
        }

        if self.ultimate.barrier_visible() {
            assert!(self.player.y == HEIGHT - 1);
            const { assert!(HEIGHT > 1); }
            self.game_grid[self.player.y - 1].fill(BARRIER_ICON);
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
            // "<h/H>".blue().bold(),
            format!("<{}/{}>", keybinds::LEFT, keybinds::DASH_LEFT)
                .blue()
                .bold(),
            " Right ".into(),
            // "<l/L>".blue().bold(),
            format!("<{}/{}>", keybinds::RIGHT, keybinds::DASH_RIGHT)
                .blue()
                .bold(),
            " Up/Down ".into(),
            format!("<{}/{}>", keybinds::UP, keybinds::DOWN)
                .blue()
                .bold(),
            " Ult ".into(),
            format!("<{}>", keybinds::label(keybinds::ULTIMATE))
                .blue()
                .bold(),
            " Quit ".into(),
            // "<q> ".blue().bold(),
            format!("<{}>", keybinds::QUIT).blue().bold(),
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
                            BARRIER_ICON => Span::from(c.to_string()).yellow(),
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

        let filled = (self.ultimate.progress() * ULT_BAR_WIDTH as f64).round() as usize;
        let bar = format!(
            "[{}{}]",
            "█".repeat(filled),
            "░".repeat(ULT_BAR_WIDTH - filled)
        );
        let ult_line = Line::from(match self.ultimate {
            Ultimate::Charging(_) => vec!["  Ult: ".into(), bar.yellow()],
            Ultimate::Ready => vec![
                "  Ult: ".into(),
                bar.green().bold(),
                " READY".green().bold(),
            ],
            Ultimate::Active(left) => vec![
                "  Ult: ".into(),
                bar.cyan(),
                format!(" {:.1}s", left.as_secs_f64()).cyan().bold(),
            ],
        });

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
            ult_line,
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
