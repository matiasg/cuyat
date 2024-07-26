use std::f32::consts::PI;
use std::rc::Rc;
use std::{cell::RefCell, fs};

use cursive::{
    event::{Event, EventResult},
    theme::{Color, ColorStyle},
    Printer, Vec2, View,
};
use nalgebra::{DVector, Dyn, OMatrix, OVector, SVector, UnitQuaternion, U3};
use regex::Regex;

type SkyMat = OMatrix<f32, Dyn, U3>;
pub type Star = SVector<f32, 3>;
type Position = SVector<f32, 3>;
type Fpp = SVector<f32, 2>; // Focal Plane Point
pub type FPStars = Vec<(Fpp, Brightness, String)>;

/// Star (position), Brightness, Name
type StBrNm = (Star, Brightness, String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Brightness {
    brightness: f32, // expected to be between 0 and 1
}
impl Brightness {
    const MAX_MAG: f32 = -1.46f32;

    fn for_magnitude(m: f32) -> Self {
        let brightness: f32 = 0.01f32.powf((m - Self::MAX_MAG) / 5.0);
        Self { brightness }
    }
    fn new(b: f32) -> Self {
        Self { brightness: b }
    }
}

#[derive(Clone)]
pub struct Sky {
    stars: Vec<StBrNm>,
}

impl Sky {
    pub fn from(stars: Vec<StBrNm>) -> Self {
        Self { stars }
    }

    pub fn from_line(line: &str, sbn_re: &Regex) -> StBrNm {
        let sbn = sbn_re.captures(line).unwrap();

        let name = String::from(sbn.get(1).unwrap().as_str());

        let rahh: u8 = sbn.get(2).unwrap().as_str().parse().unwrap();
        let ramm: u8 = sbn.get(3).unwrap().as_str().parse().unwrap();
        let rass: f32 = sbn.get(4).unwrap().as_str().parse().unwrap();
        let ra: f32 = ((rahh as f32) * 15.0 + (ramm as f32) / 4.0 + rass / 240.0).to_radians();

        let sgn: f32 = match sbn.get(5).unwrap().as_str() {
            "+" => 1.0,
            _ => -1.0,
        };
        let dedeg: u8 = sbn.get(6).unwrap().as_str().parse().unwrap();
        let demin: u8 = sbn.get(7).unwrap().as_str().parse().unwrap();
        let desec: u8 = sbn.get(8).unwrap().as_str().parse().unwrap();
        let dec: f32 =
            sgn * ((dedeg as f32) + (demin as f32) / 60.0 + (desec as f32) / 3600.0).to_radians();

        let star_pos = Star::new(ra.cos() * dec.cos(), ra.sin() * dec.cos(), dec.sin());

        let sgn: f32 = match sbn.get(9).unwrap().as_str() {
            "-" => -1.0,
            _ => 1.0,
        };
        let mag: f32 = sbn.get(10).unwrap().as_str().trim().parse().unwrap();
        let brightness = Brightness::for_magnitude(sgn * mag);
        (star_pos, brightness, name)
    }

    pub fn from_file(fname: &str) -> Self {
        let sbn_re = Regex::new("^.{7}(.{7}).{61}(\\d\\d)(\\d\\d)(\\d\\d\\.\\d)([+-])(\\d\\d)(\\d\\d)(\\d\\d).{12}([+ -])([0-9. ]{4})").unwrap();
        let input: String = fs::read_to_string(fname).unwrap();
        let input: Vec<&str> = input.trim_end().split('\n').collect();
        let stars: Vec<StBrNm> = input
            .iter()
            .map(|&line| Self::from_line(line, &sbn_re))
            .filter(|sbn| sbn.1.brightness > 0.01)
            .collect();
        Self::from(stars)
    }

    pub fn len(&self) -> usize {
        self.stars.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stars.is_empty()
    }

    pub fn seen_from(&self, pos: Position) -> Self {
        Self {
            stars: self
                .stars
                .iter()
                .map(|(s, b, n)| (*s - pos, *b, n.clone()))
                .collect(),
        }
    }

    pub fn with_attitude(&self, q: UnitQuaternion<f32>) -> Self {
        Self {
            stars: self
                .stars
                .iter()
                .map(|(s, b, n)| (q * *s, *b, n.clone()))
                .collect(),
        }
    }

    pub fn random_with_stars(n: usize) -> Self {
        let stars_positions: Vec<Star> = (0..n).map(|_| Star::new_random() * 10.0).collect();
        let brightnesses: DVector<f32> = DVector::<f32>::new_random(n);
        let names: Vec<String> = (0..n)
            // .map(|i| std::str::from_utf8(&[i as u8 + 97]).unwrap())
            .map(|i| format!("{i}"))
            .collect();
        let stars: Vec<StBrNm> = stars_positions
            .iter()
            .copied()
            .zip(brightnesses.iter().map(|&b| Brightness::new(b)))
            .zip(names.iter())
            .map(|((s, b), n)| (s, b, String::from(n)))
            .collect();
        let sky = Self { stars };
        sky.seen_from(Star::new(5.0, 5.0, 5.0))
    }
}

#[derive(Clone)]
pub struct FoV {
    half_fov_x: f32,
    half_fov_y: f32,
}

impl FoV {
    pub fn new(half_fov_x: f32, half_fov_y: f32) -> Self {
        Self {
            half_fov_x,
            half_fov_y,
        }
    }
    pub fn rescale(&self, scale: f32) -> Self {
        Self {
            half_fov_x: self.half_fov_x * scale,
            half_fov_y: self.half_fov_y * scale,
        }
    }
    fn can_be_seen(&self, b: &Brightness) -> bool {
        b.brightness / self.half_fov_x > 0.01f32.powf(0.8)
    }
    pub fn project(&self, star: &Star) -> Fpp {
        Fpp::new(
            star[0] / star[2] / self.half_fov_x,
            star[1] / star[2] / self.half_fov_y,
        )
    }
    pub fn project_sky(&self, sky: &Sky) -> FPStars {
        sky.stars
            .iter()
            .map(|(s, b, n)| (self.project(s), *b, n.clone()))
            .collect()
    }
    fn inside(x: u8, minval: u8, maxval: u8) -> bool {
        minval <= x && x <= maxval
    }
    pub fn to_screen(&self, star: &Star, maxx: u8, maxy: u8) -> Option<(u8, u8)> {
        if star[2] <= 0.0 {
            return None;
        }
        let fpp = self.project(star);
        let x = ((fpp[0] + 1.0) / 2.0 * (maxx as f32)).floor() as u8;
        let y = ((fpp[1] + 1.0) / 2.0 * (maxy as f32)).floor() as u8;

        if Self::inside(x, 0, maxx) && Self::inside(y, 0, maxy) {
            Some((x, y))
        } else {
            None
        }
    }
    pub fn project_sky_to_screen(
        &self,
        sky: Sky,
        maxx: u8,
        maxy: u8,
    ) -> Vec<Option<(u8, u8, u8, String)>> {
        sky.stars
            .iter()
            .map(|(s, b, n)| {
                let sp = self.to_screen(s, maxx, maxy);
                if sp.is_none() || !self.can_be_seen(b) {
                    None
                } else {
                    let sp = sp.unwrap();
                    let bu = 128 + (b.brightness * 128.0).floor() as u8;
                    Some((sp.0, sp.1, bu, String::from(n)))
                }
            })
            .collect()
    }

    pub fn with_angles(x_rad: f32, y_rad: f32) -> Self {
        Self {
            half_fov_x: x_rad.tan() / 2.0,
            half_fov_y: y_rad.tan() / 2.0,
        }
    }
}

pub struct Scoring {
    pub total: f32,
    pub moves: usize,
    pub games: usize,
}

impl Scoring {
    fn add_move(&mut self) {
        self.moves += 1;
    }

    fn score_and_reset(&mut self, add: f32) {
        self.total += add * (self.moves as f32 + 20.0);
        self.games += 1;
        self.moves = 0;
    }

    pub fn get_score(&self) -> f32 {
        self.total / (self.games as f32)
    }

    fn default() -> Scoring {
        Scoring {
            total: 0f32,
            moves: 0,
            games: 0,
        }
    }
}

#[derive(Clone)]
struct Options {
    show_distance: bool,
    renew_sky: bool,
}

#[derive(Clone)]
pub struct SkyView {
    pub sky: Sky,
    fov: FoV,
    q: UnitQuaternion<f32>,
    step: f32,
    margin: usize,
    scoring: Rc<RefCell<Scoring>>,
    options: Options,
}

impl SkyView {
    pub fn new(nstars: usize) -> (Self, Rc<RefCell<Scoring>>) {
        let sky = random_sky(nstars);
        Self::new_from(sky, true)
    }

    pub fn new_from(sky: Sky, renew_sky: bool) -> (Self, Rc<RefCell<Scoring>>) {
        let fov = FoV::new(2.0, 2.0);
        let scoring = Rc::new(RefCell::new(Scoring::default()));
        let options = Options {
            show_distance: false,
            renew_sky,
        };
        let q = random_quaternion();
        (
            Self {
                sky,
                fov,
                q,
                step: 0.125,
                margin: 1,
                scoring: Rc::clone(&scoring),
                options,
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
            .iter()
            .filter(|p| p.is_some())
        {
            let (px, py, b, n) = fps.as_ref().unwrap();
            // let b = fps.as_ref().unwrap().2;
            let style = ColorStyle::new(Color::Rgb(*b, *b, *b), Color::Rgb(0, 0, 32));
            p.with_color(style, |printer| {
                printer.print((*px, *py), n);
            });
        }
    }

    fn distance(&self) -> f32 {
        let (roll, pitch, yaw) = self.q.euler_angles();
        (roll.powi(2) + pitch.powi(2) + yaw.powi(2)).sqrt()
    }

    fn restart(&mut self) {
        (*self.scoring)
            .borrow_mut()
            .score_and_reset(self.distance());
        if self.options.renew_sky {
            self.sky = random_sky(self.sky.len());
        }
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

        let left = cursive::Vec2::new(0, 2);
        let left_printer = p.offset(left);
        self.draw_portion(self.q, &left_printer, x_mid, y_max);

        let style = ColorStyle::new(Color::Rgb(20, 200, 200), Color::Rgb(0, 0, 0));
        for y in 0..y_max {
            p.with_color(style, |printer| printer.print((x_mid, y), "|"))
        }

        let right = cursive::Vec2::new(x_mid as usize + self.margin, 2);
        let right_printer = p.offset(right);
        self.draw_portion(UnitQuaternion::default(), &right_printer, x_mid, y_max);

        let distance = if self.options.show_distance {
            format!("distance: {:.6}, ", self.distance())
        } else {
            String::from("")
        };
        let status_bar = format!(
            "{}Step: {:.4}, zoom: {:.3}. Moves: {}, games: {}, score: {:.6}",
            distance,
            self.step,
            self.fov.half_fov_x,
            (*self.scoring).borrow().moves,
            (*self.scoring).borrow().games,
            (*self.scoring).borrow().get_score(),
        );

        p.with_color(style, |printer| printer.print((1, 0), status_bar.as_str()));
    }
    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(121, 32)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('p') => {
                self.rotate(-1.0, 0.0, 0.0);
            }
            Event::Char('P') => {
                self.rotate(1.0, 0.0, 0.0);
            }
            Event::Char('y') => {
                self.rotate(0.0, 1.0, 0.0);
            }
            Event::Char('Y') => {
                self.rotate(0.0, -1.0, 0.0);
            }
            Event::Char('r') => {
                self.rotate(0.0, 0.0, -1.0);
            }
            Event::Char('R') => {
                self.rotate(0.0, 0.0, 1.0);
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
            Event::Char('q') => {
                self.restart();
                return EventResult::Ignored;
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

fn random_quaternion() -> nalgebra::Unit<nalgebra::Quaternion<f32>> {
    let rpy: OVector<f32, U3> = OVector::<f32, U3>::new_random() * 2.0 * PI;
    UnitQuaternion::from_euler_angles(rpy[0], rpy[1], rpy[2])
}

fn random_sky(nstars: usize) -> Sky {
    Sky::random_with_stars(nstars)
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use regex::Regex;
    use std::f32::consts::PI;

    use nalgebra::UnitQuaternion;

    use crate::{Brightness, FoV, Fpp, Position, Sky, StBrNm, Star};

    fn stars() -> Vec<StBrNm> {
        vec![
            (
                Star::new(0.0, 1.0, 2.0),
                Brightness::new(0.5),
                String::from("a"),
            ),
            (
                Star::new(3.0, 4.0, 5.0),
                Brightness::new(0.25),
                String::from("b"),
            ),
        ]
    }
    #[test]
    fn test_sky() {
        let sky = Sky::from(stars());
        assert_eq!(sky.len(), 2);
        let pos = Position::new(-1.0, -2.0, -3.0);
        let from_pos = sky.seen_from(pos);
        assert_eq!(from_pos.len(), 2);
        assert_eq!(
            from_pos.stars,
            vec![
                (
                    Star::new(1.0, 3.0, 5.0),
                    Brightness::new(0.5),
                    String::from("a")
                ),
                (
                    Star::new(4.0, 6.0, 8.0),
                    Brightness::new(0.25),
                    String::from("b")
                )
            ]
        );
        let q = UnitQuaternion::from_euler_angles(0.0, 0.0, PI / 2.0);
        let rotated = from_pos.with_attitude(q);
        assert_eq!(rotated.len(), 2);
        assert!((rotated.stars[0].0 - Star::new(-3.0, 1.0, 5.0)).norm() < 1e-5);
        assert!((rotated.stars[1].0 - Star::new(-6.0, 4.0, 8.0)).norm() < 1e-5);
    }

    #[test]
    fn test_fov() {
        let fov = FoV::new(1.0, 2.5);
        let proj_stars = fov.project_sky(&Sky::from(stars()));
        assert!((proj_stars[0].0 - Fpp::new(0.0, 0.2)).norm() < 1e-5);
        assert!((proj_stars[1].0 - Fpp::new(0.6, 0.32)).norm() < 1e-5);
    }

    #[test]
    fn test_from_line() {
        let sbn_re = Regex::new("^.{7}(.{7}).{61}(\\d\\d)(\\d\\d)(\\d\\d\\.\\d)([+-])(\\d\\d)(\\d\\d)(\\d\\d).{12}([+ -])([0-9. ]{4})").unwrap();
        let bet_line = "2061 58Alp OriBD+07 1055  39801113271 224I   4506  Alp Ori  054945.4+072319055510.3+072425199.79-08.96 0.50  +1.85 +2.06 +1.28   M1-2Ia-Iab        e+0.026+0.009 +.005+021SB         9.9 174.4AE   6*";
        let sir_line = "2491  9Alp CMaBD-16 1591  48915151881 257I   5423           064044.6-163444064508.9-164258227.22-08.88-1.46   0.00 -0.05 -0.03   A1Vm               -0.553-1.205 +.375-008SBO    13 10.3  11.2AB   4*";
        let betelgeuse = Sky::from_line(bet_line, &sbn_re);
        let exp_bet = Star::new(0.0208902, 0.9914355, 0.1289158);
        (0..3)
            .for_each(|i| assert_relative_eq!(betelgeuse.0[i], exp_bet[i], epsilon = f32::EPSILON));
        assert_eq!(betelgeuse.1, Brightness::for_magnitude(0.5));
        assert_eq!(betelgeuse.2, "Alp Ori");
        let sirius = Sky::from_line(sir_line, &sbn_re);
        let exp_sir = Star::new(-0.18745413, 0.93921775, -0.2876299);
        (0..3).for_each(|i| assert_relative_eq!(sirius.0[i], exp_sir[i], epsilon = f32::EPSILON));
        assert_eq!(sirius.1, Brightness::for_magnitude(-1.46));
        assert_eq!(sirius.2, "Alp CMa");
    }
}
