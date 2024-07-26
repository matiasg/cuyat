use cuyat::{Sky, SkyView};

fn main() {
    let sky = Sky::from_file("bsc5.dat");
    let (sky_view, score_rc) = SkyView::new_from(sky, false);
    // let (sky_view, score_rc) = SkyView::new(24);
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    let score = (*score_rc).borrow();
    println!(
        "


        ========
        total: {:.6}
        games: {}
        --------
        score: {:.6}
        ========

        ",
        score.total,
        score.games,
        score.get_score()
    );
}
