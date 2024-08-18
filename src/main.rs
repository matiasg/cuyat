use std::{cell::RefCell, env, rc::Rc};

use cuyat::{
    gview::GSkyView,
    view::{Scoring, SkyView},
};
use miniquad::conf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let scoring = Rc::new(RefCell::new(Scoring::default()));
    match args[1].as_str() {
        "cli" => {
            let sky_view = SkyView::new(Some(String::from("bsc5.csv")), 400, Rc::clone(&scoring));
            cursive_window(sky_view);
        }
        "gui" => {
            let gsky_view = GSkyView::new();
            graphics_window(gsky_view);
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

fn graphics_window(gsky_view: GSkyView) {
    miniquad::start(conf::Conf::default(), || Box::new(gsky_view));
}

fn cursive_window(sky_view: SkyView) {
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
