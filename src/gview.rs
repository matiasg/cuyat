use miniquad::{EventHandler, KeyMods};

use crate::sky::{random_quaternion, Sky};

#[derive(Debug)]
pub struct GSkyView {
    sky: Sky,
}

impl GSkyView {
    pub fn new() -> Self {
        let catalog = Some("bsc5.csv".to_string());
        let target_q = random_quaternion();
        let nstars: usize = 1200;
        let sky = Sky::new(&catalog, nstars).with_attitude(target_q);
        Self { sky }
    }
}

impl EventHandler for GSkyView {
    fn update(&mut self) {}
    fn draw(&mut self) {}
    fn char_event(&mut self, key: char, km: KeyMods, _: bool) {}
}
