use std::{collections::HashMap, io, time::{Duration, Instant}};
use rand::distr::{self, Distribution};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text, Span},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

const WIDTH: usize = 50;
const HEIGHT: usize = 20; 

type GameGrid = [[char; WIDTH]; HEIGHT];

const TICKS_MS: u64 = 15;
const TICK_RATE: Duration = Duration::from_millis(TICKS_MS); // ~66.67 FPS

const BG_ICON: char = '.';
const PLAYER_ICON: char = '@';
const OBS_ICON: char = '■';

// Obstacle spawning
const SPAWN_AFTER_MS: Duration = Duration::from_millis(60);
const INIT_MOVE_AFTER: Duration = Duration::from_millis(110);
const INCREASE_SPEED_BY: Duration = Duration::from_millis(10);
const INIT_SPEEDUP_AFTER: Duration = Duration::from_millis(1000);
const SLOW_SPEEDUP: Duration = Duration::from_millis(100);
const MIN_MOVE_AFTER: Duration = Duration::from_millis(TICKS_MS + 2);

#[derive(Debug)]
pub struct App {
    game_grid: GameGrid,
    exit: bool,
    game_over: bool,

    player: Player,

    // Obstacles
    obstacles: HashMap<u64, Obstacle>, // { id: Obs }
    obs_id: u64,
    obs_move_after: Duration,
    speedup_after: Duration,
    time_after_spawn: Duration,
    time_after_speedup: Duration,

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

            player: Player {
                x: WIDTH / 2,
                y: HEIGHT - 1,
                ..Default::default()
            },

            obstacles: HashMap::new(),
            obs_id: 0,
            obs_move_after: INIT_MOVE_AFTER,
            speedup_after: INIT_SPEEDUP_AFTER,
            time_after_spawn: Duration::ZERO,
            time_after_speedup: Duration::ZERO,

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
                    && (key_event.kind == KeyEventKind::Press 
                        || key_event.kind == KeyEventKind::Repeat) {
                self.handle_key_event(key_event);
            }

            if !self.game_over && last_update.elapsed() >= TICK_RATE {
                self.update_game(last_update.elapsed());
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

    fn add_obstacle(&mut self, obs: Obstacle) {
        self.obstacles.insert(self.obs_id, obs);
        self.obs_id += 1;
    }

    fn try_spawn_obstacle(&mut self, dt: Duration) {
        self.time_after_spawn += dt;
        self.time_after_speedup += dt;
        if self.time_after_spawn >= SPAWN_AFTER_MS {
            self.time_after_spawn = Duration::ZERO;
            let x = self.distr.sample(&mut self.rng);
            self.add_obstacle(
                // Obstacle::new(x, 0, self.obs_move_after, Shape::Unit)
                Obstacle::new(x, 0, self.obs_move_after, Shape::Brick { width: 5, height: 2 })
            );
        }

        if self.time_after_speedup >= self.speedup_after {
            self.time_after_speedup = Duration::ZERO;

            if let Some(value) = self.obs_move_after.checked_sub(INCREASE_SPEED_BY) 
                    && value >= MIN_MOVE_AFTER {
                self.obs_move_after = value;
            }
            
            self.speedup_after += SLOW_SPEEDUP;
        }
    }

    fn move_obstacles(&mut self, dt: Duration) {
        // Clean obstacles that are OBS
        self.obstacles.retain(|_, obs| {
            obs.move_obstacle(dt);
            // Check out of bounds
            obs.cells.iter().any(|&(_x, y)| {
                y < HEIGHT
            })
        });
    }

    fn move_player(&mut self) {
        match self.player.move_state {
            PlayerMoveState::MovingLeft => self.go_left(),
            PlayerMoveState::MovingRight => self.go_right(),
            PlayerMoveState::Idle => (),
        }

        self.player.move_state = PlayerMoveState::Idle;
    }

    fn check_collision(&self) -> bool {
        for obs in self.obstacles.values() {
            // if obs.x == self.player.x && obs.y == self.player.y {
            if obs.cells.contains(&(self.player.x, self.player.y)) {
                return true;
            }
        }
        false
    }

    fn update_game(&mut self, dt: Duration) {
        self.move_obstacles(dt);
        self.move_player();
        self.try_spawn_obstacle(dt);

        if self.check_collision() {
            self.game_over = true;
        }

        self.update_grid();

        self.elapsed_time += dt;
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
            // KeyCode::Char('h') => self.go_left(),
            // KeyCode::Char('l') => self.go_right(),
            KeyCode::Char('h') => self.player.move_state = PlayerMoveState::MovingLeft,
            KeyCode::Char('l') => self.player.move_state = PlayerMoveState::MovingRight,
            // KeyCode::Char('H') => self.dash_left(),
            // KeyCode::Char('L') => self.dash_right(),
            _ => {}
        }
    }

    fn get_final_score(&self) -> u128 { self.elapsed_time.as_millis() / 10 }


    fn go_left(&mut self) {
        self.player.x = if self.player.x == 0 { WIDTH - 1 } else { self.player.x - 1 };
    }

    fn go_right(&mut self) {
        self.player.x = (self.player.x + 1) % WIDTH;
    }

    fn clear_board(&mut self) {
        for row in &mut self.game_grid { row.fill(BG_ICON); }
    }

    fn update_grid(&mut self) {
        self.clear_board();

        self.game_grid[self.player.y][self.player.x] = PLAYER_ICON;
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
            "<h>".blue().bold(),
            " Right ".into(),
            "<l>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let game_over_text = Text::from(vec![
            Line::from("Game Over!"),
            Line::from(vec![
                "Final Score: ".into(),
                self.get_final_score().to_string().yellow(),
            ])
        ]);

        // 2D array -> vector of lines
        let mut lines: Vec<Line> = self.game_grid
            .iter()
            .map(|row| {
                Line::from(
                    row.iter()
                        .map(|&c| {
                            match c {
                                OBS_ICON => Span::from(c.to_string()).red(),
                                PLAYER_ICON => Span::from(c.to_string()).green(),
                                BG_ICON => Span::from(c.to_string()).blue(),
                                _ => Span::from(c.to_string()),
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        lines.extend(vec![
            Line::from(""),
            Line::from(vec![
                "Score: ".into(),
                format!("{:>5}", self.get_final_score()).yellow()
            ]),
            Line::from(vec![
                "Speed: ".into(),
                format!("{:>5}", self.obs_move_after.as_millis()).blue()
            ]),
        ]);

        let game_text = Text::from(lines);
        let out_text = if !self.game_over { game_text } else { game_over_text };

        Paragraph::new(out_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Debug, Default)]
enum PlayerMoveState {
    #[default]
    Idle,
    MovingLeft,
    MovingRight,
}


#[derive(Debug, Default)]
struct Player {
    x: usize,
    y: usize,
    move_state: PlayerMoveState,
}

#[derive(Debug, Default)]
enum Shape {
    #[default]
    Unit,
    Triangle, 
    Brick{ width: usize, height: usize },
}


#[derive(Debug, Default)]
struct Obstacle {
    cells: Vec<(usize, usize)>,
    // shape: Shape, 
    move_after: Duration,
    time_since_move: Duration,
}

impl Obstacle {
    fn new(x: usize, y: usize, move_after: Duration, shape: Shape) -> Self {
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
            // shape,
            ..Default::default()
        }

    }

    fn move_obstacle(&mut self, dt: Duration) {
        self.time_since_move += dt;
        if self.time_since_move >= self.move_after {
            self.time_since_move = Duration::ZERO;
            for (_x, y) in &mut self.cells { *y += 1; }
        }
    }

    fn render(&self, game_grid: &mut GameGrid) {
        for (x, y) in &self.cells {
            if *y < HEIGHT && *x < WIDTH {
                game_grid[*y][*x] = OBS_ICON
            }
        }
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

