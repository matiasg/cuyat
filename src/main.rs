use cursive::event::{Event, EventResult};
use cursive::theme::Color;
use cursive::theme::ColorStyle;
use cursive::view::View;
use cursive::{Printer, Vec2};
use nalgebra::UnitQuaternion;

use cuyat::{FoV, Sky, Star};

#[derive(Clone)]
struct SkyView {
    pub sky: Sky,
    fov: FoV,
    q: UnitQuaternion<f32>,
}

impl SkyView {
    fn new(nstars: usize) -> Self {
        let q: UnitQuaternion<f32> = UnitQuaternion::default();
        let sky = Sky::random_with_stars(nstars);
        let fov = FoV::new(2.0, 2.0);
        Self { sky, fov, q }
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.q = UnitQuaternion::from_euler_angles(x * 0.1, y * 0.1, z * 0.1) * self.q;
    }
}

impl View for SkyView {
    fn draw(&self, p: &Printer) {
        let x_max = p.size.x as u8;
        let y_max = p.size.y as u8;

        let style = ColorStyle::new(Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 64));

        for (i, fps) in self
            .fov
            .project_sky_to_screen(self.sky.with_attitude(self.q), x_max, y_max)
            .iter()
            .enumerate()
        {
            p.with_color(style, |printer| {
                printer.print(
                    (fps.0, fps.1),
                    std::str::from_utf8(&[i as u8 + 97]).unwrap(),
                );
            });
        }
    }
    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(60, 32)
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
