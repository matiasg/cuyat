use std::f32::consts::PI;

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
}

impl SkyView {
    fn new(nstars: usize) -> Self {
        let rpy: OVector<f32, U3> = OVector::<f32, U3>::new_random() * 2.0 * PI;
        let q = UnitQuaternion::from_euler_angles(rpy[0], rpy[1], rpy[2]);
        let sky = Sky::random_with_stars(nstars);
        let fov = FoV::new(2.0, 2.0);
        let step = 0.1;
        Self {
            sky,
            fov,
            q,
            step,
            margin: 1,
        }
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.q =
            UnitQuaternion::from_euler_angles(x * self.step, y * self.step, z * self.step) * self.q;
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
                format!("distance: {:.6}. Step: {:.4}", self.distance(), self.step).as_str(),
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
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed(None)
    }
}

fn main() {
    let sky_view: SkyView = SkyView::new(12);
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
