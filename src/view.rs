use std::{cell::RefCell, rc::Rc};

use cursive::{
    event::{Event, EventResult},
    theme::{Color, ColorStyle},
    Printer, Vec2, View,
};
use nalgebra::UnitQuaternion;

use crate::sky::{random_quaternion, FoV, Sky};

#[derive(Clone)]
struct Options {
    show_distance: bool,
    show_star_names: bool,
    catalog_filename: Option<String>,
    nstars: usize,
}

#[derive(Clone)]
pub struct SkyView {
    pub sky: Sky,
    fov: FoV,
    q: UnitQuaternion<f32>,
    step: f32,
    scoring: Rc<RefCell<Scoring>>,
    options: Options,
    headers: usize,
    vmargin: usize,
}

impl SkyView {
    pub fn new(catalog: Option<String>, nstars: usize) -> (Self, Rc<RefCell<Scoring>>) {
        let sky = Sky::new(&catalog, nstars);
        let options = Options {
            show_distance: false,
            show_star_names: true,
            catalog_filename: catalog,
            nstars,
        };
        let fov = FoV::new(2.0, 2.0);
        let scoring = Rc::new(RefCell::new(Scoring::default()));
        let q = random_quaternion();
        (
            Self {
                sky,
                fov,
                q,
                step: 0.125,
                scoring: Rc::clone(&scoring),
                options,
                headers: 2,
                vmargin: 1,
            },
            scoring,
        )
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.q =
            UnitQuaternion::from_euler_angles(x * self.step, y * self.step, z * self.step) * self.q;
        (*self.scoring).borrow_mut().add_move();
    }

    fn draw_portion(&self, quat: UnitQuaternion<f32>, p: &Printer, x_max: u8, y_max: u8) {
        for fps in self
            .fov
            .project_sky_to_screen(self.sky.with_attitude(quat), x_max, y_max)
            .into_iter()
            .flatten()
        {
            let (px, py, b, n) = fps;
            let style = ColorStyle::new(Color::Rgb(b, b, b), Color::Rgb(0, 0, 32));
            let id = if self.options.show_star_names {
                n.as_str()
            } else {
                "*"
            };
            p.with_color(style, |printer| {
                printer.print((px, py), id);
            });
        }
    }

    fn draw_header(&self, p: &Printer, style: ColorStyle) {
        let distance = if self.options.show_distance {
            format!("distance: {:.6}, ", self.distance())
        } else {
            String::from("")
        };
        let header_1 = format!(
            "{}Step: {:.4}, zoom: {:.3}. Moves: {}, games: {}, score: {:.6}",
            distance,
            self.step,
            self.fov.zoom(),
            (*self.scoring).borrow().moves,
            (*self.scoring).borrow().games,
            (*self.scoring).borrow().get_score(),
        );
        p.with_color(style, |printer| printer.print((1, 0), header_1.as_str()));
        let header_2 = format!("stars: {}", self.options.nstars,);
        p.with_color(style, |printer| printer.print((1, 1), header_2.as_str()));
    }

    fn distance(&self) -> f32 {
        let (roll, pitch, yaw) = self.q.euler_angles();
        (roll.powi(2) + pitch.powi(2) + yaw.powi(2)).sqrt()
    }

    fn restart(&mut self) {
        (*self.scoring)
            .borrow_mut()
            .score_and_reset(self.distance());
        self.sky = Sky::new(&self.options.catalog_filename, self.options.nstars);
        self.q = random_quaternion();
        self.step = 0.125;
    }

    fn zoom(&mut self, direction: f32) {
        let fov = self.fov.rescale(direction);
        self.fov = fov;
    }
}

impl View for SkyView {
    fn draw(&self, p: &Printer) {
        let x_max = p.size.x as u8;
        let x_mid = x_max / 2;
        let y_max = p.size.y as u8;

        let left = cursive::Vec2::new(0, self.headers);
        let left_printer = p.offset(left);
        self.draw_portion(self.q, &left_printer, x_mid, y_max);

        let style = ColorStyle::new(Color::Rgb(20, 200, 200), Color::Rgb(0, 0, 0));
        for y in 0..y_max {
            p.with_color(style, |printer| printer.print((x_mid, y), "|"))
        }

        let right = cursive::Vec2::new(x_mid as usize + self.vmargin, self.headers);
        let right_printer = p.offset(right);
        self.draw_portion(UnitQuaternion::default(), &right_printer, x_mid, y_max);

        let header_offset = cursive::Vec2::new(1, 0);
        let header_printer = p.offset(header_offset);
        self.draw_header(&header_printer, style);
    }
    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(121, 32)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        // TODO: add key for changing random/real stars
        match event {
            Event::Char('P') => {
                self.rotate(-1.0, 0.0, 0.0);
            }
            Event::Char('p') => {
                self.rotate(1.0, 0.0, 0.0);
            }
            Event::Char('Y') => {
                self.rotate(0.0, 1.0, 0.0);
            }
            Event::Char('y') => {
                self.rotate(0.0, -1.0, 0.0);
            }
            Event::Char('R') => {
                self.rotate(0.0, 0.0, 1.0);
            }
            Event::Char('r') => {
                self.rotate(0.0, 0.0, -1.0);
            }
            Event::Char('s') => {
                self.step /= 2.0;
            }
            Event::Char('S') => {
                self.step *= 2.0;
            }
            Event::Char('Z') => {
                self.zoom(1.25);
            }
            Event::Char('z') => {
                self.zoom(0.8);
            }
            Event::Char(' ') => {
                self.restart();
            }
            Event::Char('d') => {
                self.options.show_distance = !self.options.show_distance;
            }
            Event::Char('n') => {
                self.options.show_star_names = !self.options.show_star_names;
            }
            Event::Char('c') => {
                self.options.catalog_filename = match self.options.catalog_filename {
                    None => Some(String::from("bsc5.csv")),
                    Some(_) => None,
                };
            }
            Event::Char('v') => {
                self.options.nstars = (self.options.nstars as f32 * 0.8) as usize;
            }
            Event::Char('V') => {
                self.options.nstars = (self.options.nstars as f32 * 1.25) as usize;
            }
            Event::Char('q') => {
                self.restart();
                return EventResult::Ignored;
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

#[derive(Debug)]
pub struct Scoring {
    pub total: f32,
    pub moves: usize,
    pub games: usize,
    pub counted_moves: usize,
}

impl Scoring {
    fn add_move(&mut self) {
        self.moves += 1;
    }

    fn score_and_reset(&mut self, add: f32) {
        self.total += add * (self.moves as f32 + 20.0);
        self.games += 1;
        self.counted_moves += self.moves;
        self.moves = 0;
    }

    pub fn get_score(&self) -> f32 {
        self.total / (self.games as f32)
    }

    fn default() -> Scoring {
        Scoring {
            total: 0f32,
            moves: 0,
            counted_moves: 0,
            games: 0,
        }
    }
}
