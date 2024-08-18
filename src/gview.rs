use std::{cell::RefCell, rc::Rc};

use macroquad::prelude::*;
use nalgebra::UnitQuaternion;

use crate::{
    sky::{quat_coords_str, random_quaternion, FoV, Sky},
    view::{Options, Scoring},
};

pub struct GSkyView {
    pub sky: Sky,
    fov: FoV,
    target_q: UnitQuaternion<f32>,
    real_q: UnitQuaternion<f32>,
    step: f32,
    scoring: Rc<RefCell<Scoring>>,
    options: Options,
}

impl GSkyView {
    pub fn new(scoring: Rc<RefCell<Scoring>>) -> Self {
        let catalog = Some("bsc5.csv".to_string());
        let target_q = random_quaternion();
        let nstars: usize = 1200;
        let sky = Sky::new(&catalog, nstars).with_attitude(target_q);
        let options = Options {
            show_distance: false,
            show_star_names: true,
            catalog_filename: catalog,
            nstars,
            show_help: false,
        };
        let fov = FoV::new(2.0, 2.0);
        let real_q = random_quaternion();
        Self {
            sky,
            fov,
            target_q,
            real_q,
            step: 0.5,
            scoring: Rc::clone(&scoring),
            options,
        }
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.real_q =
            UnitQuaternion::from_euler_angles(x * self.step, y * self.step, z * self.step)
                * self.real_q;
        (*self.scoring).borrow_mut().add_move();
    }

    fn draw_portion(&self, quat: UnitQuaternion<f32>, x_max: u8, y_max: u8) {
        for fps in self
            .fov
            .project_sky_to_screen(self.sky.with_attitude(quat), x_max, y_max)
            .into_iter()
            .flatten()
        {
            let (px, py, b, n) = fps;
            let id = if self.options.show_star_names {
                n.as_str()
            } else {
                "*"
            };
            let px = (px as f32) / (x_max as f32) * screen_width();
            let py = (py as f32) / (y_max as f32) * screen_height();
            draw_circle(px, py, 4.0, WHITE);
        }
    }
}

#[macroquad::main("Sky")]
pub async fn main() {
    let scoring = Rc::new(RefCell::new(Scoring::default()));
    let mut view = GSkyView::new(scoring);

    loop {
        clear_background(BLACK);
        view.draw_portion(view.real_q, 80, 80);

        let sign = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let sign_step: f32 = if sign { view.step } else { -view.step };
        if is_key_down(KeyCode::P) {
            view.rotate(-sign_step, 0.0, 0.0)
        }
        if is_key_down(KeyCode::Y) {
            view.rotate(0.0, sign_step, 0.0)
        }
        if is_key_down(KeyCode::R) {
            view.rotate(0.0, 0.0, sign_step)
        }
        if is_key_down(KeyCode::S) {
            view.step *= 1.0905f32.powf(if sign { 1.0 } else { -1.0 });
        }
        if is_key_down(KeyCode::Z) {
            let scale = 1.0905f32.powf(if sign { 1.0 } else { -1.0 });
            let fov = view.fov.rescale(scale);
            view.fov = fov;
        }

        if is_key_down(KeyCode::Q) {
            break;
        }
        let header_1 = format!(
            "Stars: {}, catalog: {}. Step: {:.4}, zoom: {:.3}, moves: {}, games: {}, score: {:.6}",
            view.options.nstars,
            view.options
                .catalog_filename
                .clone()
                .unwrap_or("random".to_string()),
            view.step,
            view.fov.zoom(),
            (*view.scoring).borrow().moves,
            (*view.scoring).borrow().total.len(),
            (*view.scoring).borrow().get_score(),
        );
        draw_text(&header_1, 10.0, 10.0, 18.0, GRAY);
        draw_text(&quat_coords_str(view.real_q), 10.0, 30.0, 18.0, GRAY);

        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        // draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);

        next_frame().await
    }
}
