use std::{cell::RefCell, rc::Rc};

use cursive::{
    event::{Event, EventResult},
    theme::{Color, ColorStyle},
    Printer, Vec2, View,
};
use nalgebra::UnitQuaternion;

use crate::sky::{quat_coords_str, random_quaternion, FoV, Sky};

#[derive(Clone)]
pub struct Options {
    pub(crate) show_distance: bool,
    pub(crate) show_star_names: bool,
    pub(crate) catalog_filename: Option<String>,
    pub(crate) nstars: usize,
    pub(crate) show_help: bool,
    pub(crate) only_target: bool,
}

#[derive(Clone)]
pub struct SkyView {
    pub sky: Sky,
    fov: FoV,
    target_q: UnitQuaternion<f32>,
    real_q: UnitQuaternion<f32>,
    step: f32,
    scoring: Rc<RefCell<Scoring>>,
    options: Options,
    headers: usize,
    vmargin: usize,
}

impl SkyView {
    pub fn new(catalog: Option<String>, nstars: usize, scoring: Rc<RefCell<Scoring>>) -> Self {
        let target_q = random_quaternion();
        let sky = Sky::new(&catalog, nstars).with_attitude(target_q);
        let options = Options {
            show_distance: false,
            show_star_names: true,
            catalog_filename: catalog,
            nstars,
            show_help: false,
            only_target: false,
        };
        let fov = FoV::new(2.0, 2.0);
        let real_q = random_quaternion();
        Self {
            sky,
            fov,
            target_q,
            real_q,
            step: 0.125,
            scoring: Rc::clone(&scoring),
            options,
            headers: 3,
            vmargin: 1,
        }
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.real_q =
            UnitQuaternion::from_euler_angles(x * self.step, y * self.step, z * self.step)
                * self.real_q;
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
        let header_1 = format!(
            "Stars: {}, catalog: {}. Step: {:.4}, zoom: {:.3}, moves: {}, games: {}, score: {:.6}",
            self.options.nstars,
            self.options
                .catalog_filename
                .clone()
                .unwrap_or("random".to_string()),
            self.step,
            self.fov.zoom(),
            (*self.scoring).borrow().moves,
            (*self.scoring).borrow().total.len(),
            (*self.scoring).borrow().get_score(),
        );
        p.with_color(style, |printer| printer.print((1, 0), header_1.as_str()));
        let (real_q, difference, distance) = if self.options.show_distance {
            (
                format!("State:  {}", quat_coords_str(self.real_q)),
                format!(",   t/s: {}", quat_coords_str(self.target_q / self.real_q)),
                format!(",   distance: {:.6}", self.distance()),
            )
        } else {
            (String::from(""), String::from(""), String::from(""))
        };
        let header_2 = format!("Target: {}{}", quat_coords_str(self.target_q), distance);
        p.with_color(style, |printer| printer.print((1, 1), header_2.as_str()));
        let header_3 = format!("{}{}", real_q, difference);
        p.with_color(style, |printer| printer.print((1, 2), header_3.as_str()));
    }

    fn show_help(&self, p: &Printer, style: ColorStyle) {
        let help_lines = get_help_lines();
        let max_len = help_lines.iter().map(|l| l.len()).max().unwrap();
        for (i, line) in help_lines.iter().enumerate() {
            let padded_line = format!("{}{}", line, " ".repeat(max_len - line.len()));
            p.with_color(style, |printer| printer.print((0, i), padded_line.as_str()));
        }
    }

    fn distance(&self) -> f32 {
        let (roll, pitch, yaw) = (self.target_q / self.real_q).euler_angles();
        (roll.powi(2) + pitch.powi(2) + yaw.powi(2)).sqrt()
    }
    fn make_sky(&mut self) {
        self.sky = Sky::new(&self.options.catalog_filename, self.options.nstars)
            .with_attitude(self.target_q);
    }
    fn restart(&mut self) {
        (*self.scoring)
            .borrow_mut()
            .score_and_reset(self.distance());
        self.target_q = random_quaternion();
        self.make_sky();
        self.real_q = random_quaternion();
        self.step = 0.125;
    }

    fn zoom(&mut self, direction: f32) {
        let fov = self.fov.rescale(direction);
        self.fov = fov;
    }
}

pub fn get_help_lines() -> [String; 13] {
    [
        "y/Y  : yaw".to_owned(),
        "p/P  : pitch".to_owned(),
        "r/R  : roll".to_owned(),
        "z/Z  : zoom".to_owned(),
        "s/S  : scale".to_owned(),
        "d    : show/hide distance".to_owned(),
        "n    : show/hide star names".to_owned(),
        "c    : use real/random catalog".to_owned(),
        "v/V  : number of stars".to_owned(),
        "space: score and restart".to_owned(),
        "t    : show only target".to_owned(),
        "h    : show/hide this help".to_owned(),
        "q    : end playing the game".to_owned(),
    ]
}

impl View for SkyView {
    fn draw(&self, p: &Printer) {
        let x_max = p.size.x as u8;
        let x_mid = x_max / 2;
        let y_max = p.size.y as u8;

        let left = cursive::Vec2::new(0, self.headers);
        let left_printer = p.offset(left);
        self.draw_portion(self.real_q, &left_printer, x_mid, y_max);

        let style = ColorStyle::new(Color::Rgb(20, 200, 200), Color::Rgb(0, 0, 0));
        for y in 0..y_max {
            p.with_color(style, |printer| printer.print((x_mid, y), "|"))
        }

        let right = cursive::Vec2::new(x_mid as usize + self.vmargin, self.headers);
        let right_printer = p.offset(right);
        self.draw_portion(self.target_q, &right_printer, x_mid, y_max);

        let header_offset = cursive::Vec2::new(1, 0);
        let header_printer = p.offset(header_offset);
        self.draw_header(&header_printer, style);
        if self.options.show_help {
            let help_c = cursive::Vec2::new(x_mid as usize * 2 / 3 + self.vmargin, self.headers);
            let help_printer = p.offset(help_c);
            let style = ColorStyle::new(Color::Rgb(200, 200, 20), Color::Rgb(60, 60, 60));
            self.show_help(&help_printer, style);
        }
    }
    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(121, 36)
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
                    None => Some(String::from("assets/bsc5.csv")),
                    Some(_) => None,
                };
                self.restart();
            }
            Event::Char('v') => {
                self.options.nstars = (self.options.nstars as f32 * 0.8) as usize;
                self.make_sky();
            }
            Event::Char('V') => {
                self.options.nstars = (self.options.nstars as f32 * 1.25) as usize;
                self.make_sky();
            }
            Event::Char('q') => {
                self.restart();
                return EventResult::Ignored;
            }
            Event::Char('h') => {
                self.options.show_help = !self.options.show_help;
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

#[derive(Debug, Default)]
pub struct Scoring {
    pub total: Vec<f32>,
    pub moves: usize,
    pub counted_moves: usize,
}

impl Scoring {
    pub fn add_move(&mut self) {
        self.moves += 1;
    }

    pub fn score_and_reset(&mut self, add: f32) {
        self.total.push(add * (self.moves as f32 + 20.0));
        self.counted_moves += self.moves;
        self.moves = 0;
    }

    pub fn games(&self) -> usize {
        self.total.len()
    }

    pub fn get_score(&self) -> f32 {
        self.total.iter().sum::<f32>() / (self.total.len() as f32)
    }
}
