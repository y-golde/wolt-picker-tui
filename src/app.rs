use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Terminal},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};
use std::io::{stdout, Result, Stdout};

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input: String,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        App {
            terminal,
            input: String::from(""),
            should_quit: false,
        }
    }

    pub fn setup(&self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    pub fn teardown(&self) -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }
    pub fn update(&mut self) {
        if let Event::Key(key) = crossterm::event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        self.should_quit = true;
                    }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                }
            }
        }
    }
    // fn ui(&mut self, f: &mut Frame, top_panel_text: &str, side_panel_text: Vec<Line<'_>>) {}
    fn generate_sidebar_content(
        liked_category: String,
        disliked_categories: Vec<String>,
    ) -> Vec<Line<'static>> {
        let mut content: Vec<Line<'_>> = vec![];

        let liked_category_clone = liked_category.clone();
        let mut liked_span: Option<Span> = None;
        if liked_category_clone != "" {
            liked_span = Some(Span::styled(
                liked_category_clone,
                Style::default().bg(Color::Green).fg(Color::White),
            ));
        }

        let disliked_categories_clone = disliked_categories.clone();
        let mut disliked_span: Option<Span> = None;
        if disliked_categories_clone.len() > 0 {
            disliked_span = Some(Span::styled(
                format!("{:?}", disliked_categories_clone),
                Style::default().bg(Color::Red).fg(Color::White),
            ));
        }

        if liked_span.is_some() {
            content.push(liked_span.unwrap().clone().into());
        }

        if disliked_span.is_some() {
            content.push(disliked_span.unwrap().clone().into());
        }

        content
    }

    pub fn prompt_question(
        &mut self,
        question: &str,
        liked_category: &String,
        disliked_categories: &Vec<String>,
    ) -> Result<String> {
        let mut response = String::from("");
        loop {
            let disliked_catogories_clone = disliked_categories.clone();
            self.terminal.draw(|f| {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(f.size());
                let sub_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(layout[0]);

                f.render_widget(Paragraph::new(question), sub_layout[0]);
                f.render_widget(Paragraph::new(self.input.as_str()), sub_layout[1]);
                f.render_widget(
                    Paragraph::new(App::generate_sidebar_content(
                        liked_category.clone(),
                        disliked_catogories_clone,
                    )),
                    layout[1],
                );
            })?;

            self.update();

            if self.should_quit {
                self.should_quit = false;
                response = self.input.clone();
                self.input = String::from("");
                break;
            }
        }

        Ok(response)
    }
}
