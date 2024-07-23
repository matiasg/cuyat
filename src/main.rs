use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

use cursive::event::{Event, EventResult};
use cursive::theme::Color;
use cursive::theme::ColorStyle;
use cursive::view::View;
use cursive::{Printer, Vec2};
use nalgebra::{OVector, UnitQuaternion, U3};

use cuyat::{FoV, Sky};

#[derive(Clone)]
struct SkyView {
    pub sky: Sky,
    fov: FoV,
    q: UnitQuaternion<f32>,
    step: f32,
    margin: usize,
    scoring: Rc<RefCell<Scoring>>,
}

struct Scoring {
    total: f32,
    moves: usize,
}

impl Scoring {
    fn add_move(&mut self) {
        self.moves += 1;
    }

    fn score_and_reset(&mut self, add: f32) {
        self.total += add * (self.moves as f32 + 20.0);
        self.moves = 0;
    }

    fn get_score(&self) -> f32 {
        self.total
    }

    fn default() -> Scoring {
        Scoring {
            total: 0f32,
            moves: 0,
        }
    }
}

impl SkyView {
    fn new(nstars: usize) -> (Self, Rc<RefCell<Scoring>>) {
        let (q, sky) = make_random(nstars);
        let fov = FoV::new(2.0, 2.0);
        let scoring = Rc::new(RefCell::new(Scoring::default()));
        (
            Self {
                sky,
                fov,
                q,
                step: 0.1,
                margin: 1,
                scoring: Rc::clone(&scoring),
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
        let style = ColorStyle::new(Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 64));

        for (i, fps) in self
            .fov
            .project_sky_to_screen(self.sky.with_attitude(quat), x_max, y_max)
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
        {
            p.with_color(style, |printer| {
                printer.print(
                    (fps.unwrap().0, fps.unwrap().1),
                    std::str::from_utf8(&[i as u8 + 97]).unwrap(),
                );
            });
        }
    }
    fn distance(&self) -> f32 {
        let (roll, pitch, yaw) = self.q.euler_angles();
        (roll.powi(2) + pitch.powi(2) + yaw.powi(2)).sqrt()
    }

    fn restart(&mut self) {
        // self.total += self.distance() * (self.moves + 20) as f32;
        // *(*self.total).borrow_mut() += self.distance() * (self.moves + 20) as f32;
        (*self.scoring)
            .borrow_mut()
            .score_and_reset(self.distance());
        let (q, sky) = make_random(self.sky.len());
        self.q = q;
        self.sky = sky;
    }
    fn zoom(&mut self, direction: f32) {
        let fov = self.fov.rescale(direction);
        self.fov = fov;
    }
}

fn make_random(nstars: usize) -> (nalgebra::Unit<nalgebra::Quaternion<f32>>, Sky) {
    let rpy: OVector<f32, U3> = OVector::<f32, U3>::new_random() * 2.0 * PI;
    let q = UnitQuaternion::from_euler_angles(rpy[0], rpy[1], rpy[2]);
    let sky = Sky::random_with_stars(nstars);
    (q, sky)
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

        p.with_color(style, |printer| {
            printer.print(
                (1, 0),
                format!(
                    "distance: {:.6}. Step: {:.4}.  m: {}. TOTAL: {:.6}",
                    self.distance(),
                    self.step,
                    (*self.scoring).borrow().moves,
                    (*self.scoring).borrow().get_score(),
                )
                .as_str(),
            )
        });
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
            Event::Char('q') => {
                self.restart();
                return EventResult::Ignored;
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

fn main() {
    let (sky_view, total) = SkyView::new(12);
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    println!(
        "\n\n\n ====>>>> total: {:?} <<<====\n\n\n",
        (*total).borrow().get_score()
    );
}
