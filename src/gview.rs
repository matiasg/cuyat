use core::time;
use std::{cell::RefCell, rc::Rc, thread};

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
        let nstars: usize = 1200;
        let target_q = random_quaternion();
        let sky = Sky::new(&catalog, nstars).with_attitude(target_q);
        let options = Options {
            show_distance: false,
            show_star_names: true,
            catalog_filename: catalog,
            nstars,
            show_help: false,
        };
        let fov = FoV::new(2.0, 1.0);
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
    fn make_sky(&mut self) {
        self.sky = Sky::new(&self.options.catalog_filename, self.options.nstars)
            .with_attitude(self.target_q);
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
    fn draw_portion(
        &self,
        quat: UnitQuaternion<f32>,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        font: Option<&Font>,
    ) {
        let width = (x_max - x_min) * 256.0;
        let height = (y_max - y_min) * 256.0;
        for fps in self
            .fov
            .project_sky_to_screen(self.sky.with_attitude(quat), width as u8, height as u8)
            .into_iter()
            .flatten()
        {
            let (px, py, b, n) = fps;
            let px = (x_min + (px as f32) / 256.0) * screen_width();
            let py = (y_min + (py as f32) / 256.0) * screen_height();
            let b = (b as f32 - 64.0) / 192.0;
            let color = Color::new(b, b, b, 1.0);
            draw_circle(px, py, 4.0, color);
            if self.options.show_star_names {
                draw_text_ex(
                    &n,
                    px + 6.0,
                    py,
                    TextParams {
                        font_size: 16,
                        font,
                        ..Default::default()
                    },
                );
            }
        }
    }
    fn distance(&self) -> f32 {
        let (roll, pitch, yaw) = (self.target_q / self.real_q).euler_angles();
        (roll.powi(2) + pitch.powi(2) + yaw.powi(2)).sqrt()
    }
    fn restart(&mut self) {
        (*self.scoring)
            .borrow_mut()
            .score_and_reset(self.distance());
        self.target_q = random_quaternion();
        self.make_sky();
        self.real_q = random_quaternion();
        self.step = 0.5;
    }
}

#[macroquad::main("Sky")]
pub async fn main() {
    let scoring = Rc::new(RefCell::new(Scoring::default()));
    let font = load_ttf_font("assets/Piazzolla-Medium.ttf").await.unwrap();
    let mut view = GSkyView::new(scoring);

    loop {
        let sign = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let sign_step: f32 = if sign { view.step } else { -view.step };
        if is_key_down(KeyCode::P) {
            view.rotate(-sign_step, 0.0, 0.0);
        }
        if is_key_down(KeyCode::Y) {
            view.rotate(0.0, sign_step, 0.0);
        }
        if is_key_down(KeyCode::R) {
            view.rotate(0.0, 0.0, sign_step);
        }
        if is_key_pressed(KeyCode::S) {
            view.step *= 1.1892f32.powf(if sign { 1.0 } else { -1.0 });
        }
        if is_key_pressed(KeyCode::Z) {
            let scale = 1.0905f32.powf(if sign { 1.0 } else { -1.0 });
            let fov = view.fov.rescale(scale);
            view.fov = fov;
        }
        if is_key_pressed(KeyCode::N) {
            view.options.show_star_names = !view.options.show_star_names;
        }
        if is_key_pressed(KeyCode::V) {
            let mult: f32 = if sign { 1.25 } else { 0.8 };
            view.options.nstars = (view.options.nstars as f32 * mult).max(8.0) as usize;
            view.make_sky();
        }
        if is_key_pressed(KeyCode::Space) {
            view.restart();
        }

        if is_key_pressed(KeyCode::Q) {
            break;
        }

        clear_background(BLACK);
        view.draw_portion(view.real_q, 0.0, 1.0, 0.0, 0.5, Some(&font));
        draw_line(
            0.0,
            screen_height() / 2.0,
            screen_width(),
            screen_height() / 2.0,
            2.0,
            YELLOW,
        );
        view.draw_portion(view.target_q, 0.0, 1.0, 0.5, 1.0, Some(&font));

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
        draw_text(&header_1, 10.0, 20.0, 18.0, GRAY);
        draw_text(&quat_coords_str(view.real_q), 10.0, 38.0, 18.0, GRAY);

        thread::sleep(time::Duration::from_millis(50));
        next_frame().await;
    }
}
