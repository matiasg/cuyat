use std::{cell::RefCell, env, rc::Rc};

use cuyat::{
    gview::{self},
    view::{Scoring, SkyView},
};
use macroquad::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    let scoring = Rc::new(RefCell::new(Scoring::default()));
    match args[1].as_str() {
        "cli" => {
            let sky_view = SkyView::new(
                Some(String::from("assets/bsc5.csv")),
                400,
                Rc::clone(&scoring),
            );
            cursive_window(sky_view);
        }
        "gui" => {
            gview::main();
        }
        _ => {}
    };
    let score = (*scoring).borrow();
    println!(
        "


        ========
        moves: {}
        total: {:.6}
        games: {}
        --------
        score: {:.6}
        ========

        ",
        score.counted_moves,
        score.total.iter().sum::<f32>(),
        score.total.len(),
        score.get_score()
    );
}

fn cursive_window(sky_view: SkyView) {
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
