use std::{cmp, collections::HashMap, io, time::Duration};
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

const TICK_RATE: Duration = Duration::from_millis(10);

const BG_ICON: char = '.';
const PLAYER_ICON: char = '@';
const OBS_ICON: char = '■';

// Obstacle spawning
const SPAWN_AFTER_TICKS: u64 = 4;
const INIT_SPEEDUP_AFTER_TICKS: u64 = 100;
const SLOW_SPEEDUP: u64 = 1000;
const INIT_MOVE_AFTER_TICKS: u32 = 11;

#[derive(Debug)]
pub struct App {
    state: [[char; WIDTH]; HEIGHT],
    exit: bool,
    game_over: bool,

    // Player
    px: usize,

    // Obstacles
    obstacles: HashMap<u64, Obstacle>, // { id: Obs }
    obs_id: u64,
    obs_speed: u32,
    speedup_after_ticks: u64,

    elapsed_ticks: u64,

    rng: rand::prelude::ThreadRng,
    distr: distr::Uniform<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: [[BG_ICON; WIDTH]; HEIGHT],
            exit: false,
            game_over: false,

            px: WIDTH / 2,

            obstacles: HashMap::new(),
            obs_id: 0,
            obs_speed: 0,
            speedup_after_ticks: INIT_SPEEDUP_AFTER_TICKS,

            elapsed_ticks: 0,

            rng: rand::rng(),
            distr: distr::Uniform::new(0, WIDTH).unwrap(),
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.init_game();

        while !self.exit {
            // Poll for input (non‑blocking)
            if event::poll(Duration::ZERO)?
                    && let Event::Key(key_event) = event::read()?
                    && key_event.kind == KeyEventKind::Press {
                self.handle_key_event(key_event);
            }

            if !self.game_over {
                self.update_game();
            }

            terminal.draw(|frame| self.draw(frame))?;

            std::thread::sleep(TICK_RATE);
        }

        Ok(())
    }

    fn init_game(&mut self) {
        self.state[HEIGHT - 1][self.px] = PLAYER_ICON;
    }

    fn add_obstacle(&mut self, obs: Obstacle) {
        self.obstacles.insert(self.obs_id, obs);
        self.obs_id += 1;
    }

    fn try_spawn_obstacle(&mut self) {
        if self.elapsed_ticks.is_multiple_of(SPAWN_AFTER_TICKS) {
            let x = self.distr.sample(&mut self.rng);
            self.add_obstacle(
                Obstacle { 
                    x,
                    y: 0,
                    move_after_ticks: INIT_MOVE_AFTER_TICKS - self.obs_speed,
                    next_move: 0
                }
            );
        }

        if self.elapsed_ticks.is_multiple_of(self.speedup_after_ticks) {
            self.obs_speed = cmp::min(self.obs_speed + 1, INIT_MOVE_AFTER_TICKS - 1);
            self.speedup_after_ticks += SLOW_SPEEDUP;
        }
    }

    fn update_obstacles(&mut self) {
        for obs in self.obstacles.values_mut() {
            self.state[obs.y][obs.x] = BG_ICON;
            obs.update();

            if obs.y < HEIGHT {
                self.state[obs.y][obs.x] = OBS_ICON;
            }
        }

        // Clean dropped rocks
        self.obstacles.retain(|_, obs| obs.y < HEIGHT);
    }

    fn check_collision(&self) -> bool {
        for obs in self.obstacles.values() {
            if obs.x == self.px && obs.y == HEIGHT - 1 {
                return true;
            }
        }
        false
    }

    fn update_game(&mut self) {
        self.update_obstacles();
        self.try_spawn_obstacle();

        if self.check_collision() {
            self.game_over = true;
        }

        self.elapsed_ticks += 1;
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
            KeyCode::Char('h') => self.go_left(),
            KeyCode::Char('l') => self.go_right(),
            _ => {}
        }
    }

    fn get_final_score(&self) -> u64 { self.elapsed_ticks }

    fn go_left(&mut self) {
        // TODO: Refactor left and right
        self.state[HEIGHT-1][self.px] = BG_ICON;
        self.px = if self.px == 0 { WIDTH - 1 } else { self.px - 1 };
        self.state[HEIGHT-1][self.px] = PLAYER_ICON;
    }

    fn go_right(&mut self) {
        self.state[HEIGHT-1][self.px] = BG_ICON;
        self.px = (self.px + 1) % WIDTH;
        self.state[HEIGHT-1][self.px] = PLAYER_ICON;
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
        let mut lines: Vec<Line> = self.state
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

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
                "Score: ".into(),
                format!("{:>5}", self.get_final_score()).yellow()
        ]));

        let game_text = Text::from(lines);
        let out_text = if !self.game_over { game_text } else { game_over_text };

        Paragraph::new(out_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

// fn color_game_chars() {
// }

#[derive(Debug, Default)]
struct Obstacle {
    x: usize,
    y: usize,
    move_after_ticks: u32,
    next_move: u32,
}

impl Obstacle {
    fn update(&mut self) {
        if self.next_move > 0 {
            self.next_move -= 1;
            return;
        }

        self.next_move = self.move_after_ticks;
        self.y += 1;
    }
}

fn main() -> io::Result<()> { ratatui::run(|terminal| App::new().run(terminal))
}

