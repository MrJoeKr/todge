use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

const WIDTH: usize = 11;
const HEIGHT: usize = 11; 

const TICK_RATE: Duration = Duration::from_millis(100);

const PLAYER_ICON: char = '@';
const OBS_ICON: char = '■';

#[derive(Debug, Default)]
pub struct App {
    state: [[char; WIDTH]; HEIGHT],
    exit: bool,

    // Player
    px: usize,

    obs: Obstacle,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.init_game();

        while !self.exit {
            // 1. Poll for input (non‑blocking, 100 ms timeout)
            if event::poll(Duration::ZERO)? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            }

            // 2. Update game state (move obstacle, etc.)
            self.update_game();

            // 3. Draw the frame
            terminal.draw(|frame| self.draw(frame))?;

            std::thread::sleep(TICK_RATE);
        }

        // while !self.exit {
        //     terminal.draw(|frame| self.draw(frame))?;
        //     self.update_game()?;
        //
        //     std::thread::sleep(TICK_RATE);
        // }
        
        Ok(())
    }

    fn init_game(&mut self) {
        self.state = [['.'; WIDTH]; HEIGHT];
        self.px = WIDTH / 2;
        self.state[HEIGHT - 1][self.px] = PLAYER_ICON;

        self.obs = Obstacle {
            x: 4,
            y: 0,
            move_per_ticks: 8,
            next_move: 0,
        };
    }

    fn update_game(&mut self) {
        self.state[self.obs.y][self.obs.x] = '.';
        self.obs.update();
        self.state[self.obs.y][self.obs.x] = OBS_ICON;
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(TICK_RATE)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event);
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') => self.go_left(),
            KeyCode::Char('l') => self.go_right(),
            _ => {}
        }
    }

    fn go_left(&mut self) {
        // TODO: Refactor left and right
        self.state[HEIGHT-1][self.px] = '.';
        self.px = if self.px == 0 { WIDTH - 1 } else { self.px - 1 };
        self.state[HEIGHT-1][self.px] = PLAYER_ICON;
    }

    fn go_right(&mut self) {
        self.state[HEIGHT-1][self.px] = '.';
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

        // let bottom_line = String::from_iter(self.state[HEIGHT-1].iter());

        // let game_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     // self.counter.to_string().yellow(),
        // ])]);
        // let game_text = Text::from(bottom_line);
        let game_text = Text::from(self.state
            .into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
        );

        Paragraph::new(game_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Debug, Default)]
struct Obstacle {
    x: usize,
    y: usize,
    move_per_ticks: u32,
    next_move: u32,
}

impl Obstacle {
    fn update(&mut self) {
        if self.next_move > 0 {
            self.next_move -= 1;
            return;
        }

        self.next_move = self.move_per_ticks;
        self.y += 1;
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

