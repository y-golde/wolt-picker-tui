use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        canvas::{self, Canvas, Map, Points},
        Block, Borders, Paragraph, Wrap,
    },
    Frame,
};
use std::io::{stdout, Result, Stdout};

use std::rc::Rc;

use crate::controllers::WoltAPITypes::ResterauntItem;

pub struct RestaurantDisplayApp {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_quit: bool,
    address: (f32, f32),
    current_zoom: f64,
    choice_index: usize,
}

const MAX_MAP_ZOOM_OUT_DISTANCE: f64 = 20.0;
const MIN_MAP_ZOOM_OUT_DISTANCE: f64 = 0.3;

impl RestaurantDisplayApp {
    pub fn new(address: (f32, f32)) -> Self {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        RestaurantDisplayApp {
            terminal,
            should_quit: false,
            address,
            current_zoom: MAX_MAP_ZOOM_OUT_DISTANCE,
            choice_index: 0,
        }
    }

    pub fn _setup(&self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    pub fn _teardown(&mut self) -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        self.terminal.clear()?;
        Ok(())
    }

    pub fn choice_input_listener(&mut self, choices_len: &usize) -> Result<()> {
        if crossterm::event::poll(std::time::Duration::from_millis(15))? {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            self.should_quit = true;
                        }
                        KeyCode::Down => {
                            if self.choice_index == choices_len - 1 {
                                self.choice_index = 0;
                            } else {
                                self.choice_index += 1;
                            }
                        }
                        KeyCode::Up => {
                            if self.choice_index == 0 {
                                self.choice_index = choices_len - 1;
                            } else {
                                self.choice_index -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /**
     * returns (layout, sub_layout)
     */
    fn get_restaurant_display(frame: &Frame) -> (Rc<[Rect]>, Rc<[Rect]>) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());
        let sub_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        (layout, sub_layout)
    }

    fn update_zoom(&mut self, address_coordinates: (f64, f64), restaurant_coordinates: (f64, f64)) {
        let euclidean_distance = ((restaurant_coordinates.0 - address_coordinates.0).powi(2)
            + (restaurant_coordinates.1 - address_coordinates.1).powi(2))
        .sqrt();

        let calculated_min_zoom = f64::max(euclidean_distance * 10.0, MIN_MAP_ZOOM_OUT_DISTANCE);

        if self.current_zoom > calculated_min_zoom {
            self.current_zoom -= 0.2;
        }
    }

    fn get_restaurant_description(restaurant: &ResterauntItem) -> String {
        let name = &restaurant.title;
        let categories = &restaurant.filtering.filters[0].values;
        let price = &restaurant.venue.delivery_price;
        let range = &restaurant.venue.estimate_range;
        let slug = &restaurant.venue.slug;

        format!(
            "{name} \n {range}min - {price} \n {categories} \n https://wolt.com/en/isr/tel-aviv/restaurant/{slug}",
            name = name,
            categories = categories.join(", "),
            price = price,
            range = range,
            slug = slug
        )
    }

    fn get_choices_element(choices: &Vec<String>, choice_index: usize) -> Vec<Line> {
        let mut lines: Vec<Line> = vec![];

        let mut index = 0;
        for choice in choices {
            let span;
            if index == choice_index {
                span = Span::styled(format!(">> {}", choice), Style::new().on_light_yellow());
            } else {
                span = Span::styled(format!("   {}", choice), Style::new());
            }

            lines.push(Line::from(span));
            index += 1;
        }

        lines
    }

    fn render_top_section(f: &mut Frame, top_section_text: String, area: Rect) {
        f.render_widget(
            Paragraph::new(top_section_text)
                .alignment(Alignment::Center)
                .white()
                .on_light_blue()
                .wrap(Wrap { trim: false }),
            area,
        );
    }

    fn render_choices(f: &mut Frame, choices: &Vec<String>, choice_index: usize, area: Rect) {
        let choices_element = RestaurantDisplayApp::get_choices_element(&choices, choice_index);

        f.render_widget(
            Paragraph::new(choices_element).block(Block::default().borders(Borders::ALL)),
            area,
        );
    }

    fn render_map(
        f: &mut Frame,
        zoom: f64,
        address_coordinates: (f64, f64),
        restaurant_coordinates: (f64, f64),
        area: Rect,
    ) {
        let map = Map {
            resolution: canvas::MapResolution::High,
            color: Color::Cyan,
        };

        let (addr_lat, addr_lon) = address_coordinates; // yes, the coordinates are backwards...

        // Displaying the map
        f.render_widget(
            Canvas::default()
                .marker(ratatui::symbols::Marker::HalfBlock)
                .x_bounds([addr_lat - zoom, addr_lat + zoom])
                .y_bounds([addr_lon - zoom, addr_lon + zoom])
                .paint(|context| {
                    context.draw(&map);

                    context.draw(&canvas::Line::new(
                        addr_lat,
                        addr_lon,
                        restaurant_coordinates.0,
                        restaurant_coordinates.1,
                        Color::Green,
                    ));

                    context.draw(&Points {
                        color: Color::Magenta,
                        coords: &[restaurant_coordinates],
                    });

                    context.draw(&Points {
                        color: Color::White,
                        coords: &[address_coordinates],
                    });
                }),
            area,
        );
    }

    fn render_resteraunt_display(
        f: &mut Frame,
        top_section_text: String,
        choices: &Vec<String>,
        choice_index: usize,
        zoom: f64,
        address_coordinates: (f64, f64),
        restaurant_coordinates: (f64, f64),
    ) {
        let (layout, sub_layout) = RestaurantDisplayApp::get_restaurant_display(&f);

        RestaurantDisplayApp::render_top_section(f, top_section_text, sub_layout[0]);
        RestaurantDisplayApp::render_choices(f, choices, choice_index, sub_layout[1]);
        RestaurantDisplayApp::render_map(
            f,
            zoom,
            address_coordinates,
            restaurant_coordinates,
            layout[1],
        )
    }

    fn get_coordinates(&mut self, restaurant: &ResterauntItem) -> ((f64, f64), (f64, f64)) {
        let address_coordinates: (f64, f64) = (self.address.1.into(), self.address.0.into()); // yes, the coordinates are backwards...

        let restaurant_coordinates_vec = restaurant.venue.location.to_owned();
        let restaurant_coordinates = (restaurant_coordinates_vec[0], restaurant_coordinates_vec[1]);

        (address_coordinates, restaurant_coordinates)
    }

    pub fn display_restaurant_question(
        &mut self,
        question: &str,
        restaurant: &ResterauntItem,
        choices: Vec<String>,
    ) -> Result<usize> {
        self.current_zoom = MAX_MAP_ZOOM_OUT_DISTANCE;

        let choice_index: usize;

        loop {
            let restaurant_clone = restaurant.clone();
            let (address_coordinates, restaurant_coordinates) =
                self.get_coordinates(&restaurant.clone());

            self.update_zoom(address_coordinates, restaurant_coordinates);

            let restaurant_description =
                RestaurantDisplayApp::get_restaurant_description(&restaurant_clone);
            let top_section_text = format!("{} \n {}", question, restaurant_description);

            let zoom = self.current_zoom;

            self.terminal.draw(|f| {
                RestaurantDisplayApp::render_resteraunt_display(
                    f,
                    top_section_text,
                    &choices,
                    self.choice_index,
                    zoom,
                    address_coordinates,
                    restaurant_coordinates,
                )
            })?;

            self.choice_input_listener(&choices.len()).unwrap();

            if self.should_quit {
                choice_index = self.choice_index;
                self.choice_index = 0;
                self.should_quit = false;
                break;
            }
        }

        Ok(choice_index)
    }

    pub fn display_category_question(
        &mut self,
        question: &str,
        restaurant: &ResterauntItem,
        choices: Vec<String>,
    ) -> Result<usize> {
        let choice_index: usize;

        loop {
            let (address_coordinates, restaurant_coordinates) = self.get_coordinates(&restaurant);

            let zoom = self.current_zoom;

            self.terminal.draw(|f| {
                RestaurantDisplayApp::render_resteraunt_display(
                    f,
                    question.to_string(),
                    &choices,
                    self.choice_index,
                    zoom,
                    address_coordinates,
                    restaurant_coordinates,
                )
            })?;

            self.choice_input_listener(&choices.len()).unwrap();

            if self.should_quit {
                choice_index = self.choice_index;
                self.choice_index = 0;
                self.should_quit = false;
                break;
            }
        }

        Ok(choice_index)
    }
}
