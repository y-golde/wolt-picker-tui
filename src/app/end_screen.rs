use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        canvas::{self, Canvas, Map, Points},
        Block, Borders, Padding, Paragraph, Wrap,
    },
    Frame,
};
use std::io::{stdout, Result, Stdout};

pub struct EndScreenApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    acc_x: i32,
    acc_y: i32,
    rect_x: i32,
    rect_y: i32,
    border_color: Color,
}

const MODAL_WIDTH: u16 = 50;
const MODAL_HEIGHT: u16 = 15;

impl EndScreenApp {
    pub fn new() -> Self {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();

        Self {
            terminal,
            should_quit: false,
            acc_x: 1,
            acc_y: 1,
            rect_x: 0,
            rect_y: 0,
            border_color: Color::Magenta,
        }
    }

    pub fn listener(&mut self) -> Result<()> {
        if crossterm::event::poll(std::time::Duration::from_millis(15))? {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn set_random_color(&mut self) {
        let color_pool = vec![
            Color::Red,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Yellow,
            Color::White,
            Color::Green,
        ];

        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..color_pool.len());
        let random_color = color_pool[random_index];

        self.border_color = random_color
    }

    fn augment_rect(&mut self) {
        let screen_size = self.terminal.size().unwrap();

        // todo: tidy this monstrosity
        if self.acc_x > 0 {
            if self.rect_x >= i32::from(screen_size.width - MODAL_WIDTH) {
                self.acc_x *= -1;
                self.set_random_color();
            }
        } else {
            if self.rect_x <= 0 {
                self.acc_x *= -1;
                self.set_random_color();
            }
        }

        if self.acc_y > 0 {
            if self.rect_y >= i32::from(screen_size.height - MODAL_HEIGHT) {
                self.acc_y *= -1;
                self.set_random_color();
            }
        } else {
            if self.rect_y <= 0 {
                self.acc_y *= -1;
                self.set_random_color();
            }
        }

        self.rect_x += self.acc_x;
        self.rect_y += self.acc_y;
    }

    pub fn display_end_screen(&mut self, end_screen_message: String) {
        loop {
            let rect = Rect {
                x: self.rect_x as u16,
                y: self.rect_y as u16,
                width: MODAL_WIDTH,
                height: MODAL_HEIGHT,
            };

            self.terminal
                .draw(|f| {
                    let p = Paragraph::new(end_screen_message.as_str())
                        .alignment(Alignment::Center)
                        .white()
                        .on_light_blue()
                        .wrap(Wrap { trim: false })
                        .block(
                            Block::new()
                                .borders(Borders::all())
                                .border_style(Style::default().fg(self.border_color))
                                .title("You did it!")
                                .style(Style::new().bg(Color::Black)),
                        );
                    f.render_widget(p, rect);
                })
                .unwrap();
            self.augment_rect();

            self.listener().unwrap();

            if self.should_quit {
                break;
            }
        }
    }
}
