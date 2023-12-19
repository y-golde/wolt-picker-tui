use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        canvas::{self, Canvas, Map, Points},
        Paragraph, Widget, Wrap,
    },
    Frame,
};
use std::io::{stdout, Result, Stdout};

use std::rc::Rc;

use crate::controllers::WoltAPITypes::ResterauntItem;

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input: String,
    should_quit: bool,
    address: (f32, f32),
    current_zoom: f64,
}

const MAX_MAP_ZOOM_OUT_DISTANCE: f64 = 10.0;
const MIN_MAP_ZOOM_OUT_DISTANCE: f64 = 0.3;

impl App {
    pub fn new(address: (f32, f32)) -> Self {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        App {
            terminal,
            input: String::from(""),
            should_quit: false,
            address,
            current_zoom: MAX_MAP_ZOOM_OUT_DISTANCE,
        }
    }

    pub fn _setup(&self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    pub fn _teardown(&self) -> Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        if crossterm::event::poll(std::time::Duration::from_millis(25))? {
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

    pub fn display_restaurant(&mut self, restaurant: &ResterauntItem) -> Result<()> {
        self.current_zoom = MAX_MAP_ZOOM_OUT_DISTANCE;

        loop {
            let address_coordinates: (f64, f64) = (self.address.0.into(), self.address.1.into());
            let (addr_lon, addr_lat) = address_coordinates; // yes, the coordinates are backwards...
            let restaurant_clone = restaurant.clone();

            let restaurant_coordinates_vec = restaurant_clone.venue.location.to_owned();
            let restaurant_coordinates =
                (restaurant_coordinates_vec[0], restaurant_coordinates_vec[1]);

            self.update_zoom((addr_lat, addr_lon), restaurant_coordinates);

            let description = App::get_restaurant_description(&restaurant_clone);

            self.terminal.draw(|f| {
                let (layout, sub_layout) = App::get_restaurant_display(&f);

                f.render_widget(
                    Paragraph::new(description)
                        .alignment(Alignment::Center)
                        .white()
                        .on_light_blue()
                        .wrap(Wrap { trim: false }),
                    sub_layout[0],
                );
                f.render_widget(Paragraph::new(self.input.as_str()), sub_layout[1]);

                let map = Map {
                    resolution: canvas::MapResolution::High,
                    color: Color::Cyan,
                };

                let zoom = self.current_zoom;
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
                                coords: &[(addr_lat, addr_lon)],
                            });
                        }),
                    layout[1],
                ); // work on the map
            })?;

            self.update();

            if self.should_quit {
                self.should_quit = false;
                break;
            }
        }
        Ok(())
    }
}
